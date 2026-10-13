use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use codegen::{ir::RootIR, lower};
use errors::Error;
use parser::unwrap_or_report_file;
use resolver::{
    check,
    context::{Context, Primitives},
    defkinds::TypeAliasDef,
    imports::get_imports,
    resolve,
    symbols::DefKind,
};
use types::{ScopeID, TypeID};

#[derive(Debug, Clone)]
pub struct UnitInfo {
    pub file_path: PathBuf,
    pub unit_path: String,
    pub scope: ScopeID,
    pub ast: ast::Root,
    pub imports: Vec<String>,
}

#[derive(Debug)]
pub struct AnalyzedUnit {
    pub unit_path: String,
    pub ir: RootIR,
    pub scope: ScopeID,
    pub imports: Vec<String>,
}

pub struct Project {
    project_dir: PathBuf,
    ctx: Context,
    units: HashMap<String, UnitInfo>,
}

impl Project {
    pub fn new(entry: &Path) -> Result<Self, Error> {
        let abs_path = entry
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path {}: {}", entry.display(), e))?;

        let project_dir = abs_path
            .parent()
            .ok_or_else(|| format!("Failed to get parent directory of {}", abs_path.display()))?
            .to_path_buf();

        let mut ctx = Context::new(Primitives::default());
        load_primitives(&mut ctx);

        Ok(Self {
            project_dir,
            ctx,
            units: HashMap::new(),
        })
    }

    pub fn load_project(entry: &Path) -> Result<Self, Error> {
        let mut project = Project::new(entry)?;

        let root_scope = project.ctx.table.root();
        let scope = project.ctx.table.push(root_scope)?;

        project.load_unit(
            &project.project_dir.join(entry.file_name().unwrap()),
            entry.file_stem().unwrap().to_string_lossy().to_string(),
            root_scope,
            scope,
        )?;

        Ok(project)
    }

    pub fn analyze(mut self) -> Result<AnalizedProject, Error> {
        let units: Vec<UnitInfo> = self.units.into_values().collect();

        for unit in units.iter() {
            self.ctx.units.register(unit.unit_path.clone(), unit.scope);
        }

        for unit in units.iter() {
            resolve(&mut self.ctx, unit.scope, &unit.ast)?;
        }

        let mut result = Vec::new();

        for unit in units.into_iter() {
            let typed = check(&mut self.ctx, unit.scope, unit.ast)?;
            let ir = lower(typed, &mut self.ctx)?;

            result.push(AnalyzedUnit {
                unit_path: unit.unit_path.clone(),
                ir,
                scope: unit.scope,
                imports: unit.imports,
            });
        }

        Ok(AnalizedProject {
            ctx: self.ctx,
            units: result,
        })
    }

    fn load_unit(
        &mut self,
        path: &Path,
        unit_path: String,
        root_scope: ScopeID,
        scope: ScopeID,
    ) -> Result<(), Error> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

        let root = unwrap_or_report_file!(parser::parse(&source), path.to_str().unwrap(), &source);
        let imports = get_imports(&root)?;

        self.units.insert(
            unit_path.clone(),
            UnitInfo {
                file_path: path.to_path_buf(),
                unit_path: unit_path,
                scope,
                ast: root,
                imports: imports.clone(),
            },
        );

        for import in imports {
            let import_path = self.resolve_import_path(&import);
            let new_scope = self.ctx.table.push(root_scope)?;

            if !self.units.contains_key(&import) {
                self.load_unit(&import_path, import, root_scope, new_scope)?;
            }
        }

        Ok(())
    }

    fn resolve_import_path(&self, import: &str) -> PathBuf {
        self.project_dir
            .join(import.replace(".", "/"))
            .with_extension("cleo")
    }
}

pub struct AnalizedProject {
    pub ctx: Context,
    pub units: Vec<AnalyzedUnit>,
}

fn load_primitives(ctx: &mut Context) {
    use types::defs::{Signedness, TypeDef};

    let add: fn(&mut Context, &str, TypeDef) -> TypeID = |ctx, name, ty| {
        let root = ctx.table.root();
        let typeid = ctx.interner.intern(ty);

        let def = resolver::symbols::Definition {
            name: name.to_string(),
            scope: root,
            kind: DefKind::TypeAlias(TypeAliasDef {
                resolved: true,
                typeid,
            }),
        };

        ctx.table.define(root, def).unwrap();

        typeid
    };

    let primitives = Primitives {
        bool_t: add(ctx, "bool", TypeDef::Bool),
        u8_t: add(ctx, "u8", TypeDef::Int(8, Signedness::Unsigned)),
        u16_t: add(ctx, "u16", TypeDef::Int(16, Signedness::Unsigned)),
        u32_t: add(ctx, "u32", TypeDef::Int(32, Signedness::Unsigned)),
        u64_t: add(ctx, "u64", TypeDef::Int(64, Signedness::Unsigned)),
        i8_t: add(ctx, "i8", TypeDef::Int(8, Signedness::Signed)),
        i16_t: add(ctx, "i16", TypeDef::Int(16, Signedness::Signed)),
        i32_t: add(ctx, "i32", TypeDef::Int(32, Signedness::Signed)),
        i64_t: add(ctx, "i64", TypeDef::Int(64, Signedness::Signed)),
        f32_t: add(ctx, "f32", TypeDef::Float(32)),
        f64_t: add(ctx, "f64", TypeDef::Float(64)),
        usize_t: add(ctx, "usize", TypeDef::USize),
        isize_t: add(ctx, "isize", TypeDef::ISize),
        nothing_t: ctx.interner.intern(TypeDef::Empty),
    };

    ctx.set_primitives(primitives);

    let _ = add(ctx, "c_void", TypeDef::Empty);
}
