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
use types::ScopeID;

#[derive(Debug, Clone)]
pub struct UnitInfo {
    pub file_path: PathBuf,
    pub unit_path: String,
    pub scope: ScopeID,
    pub ast: ast::Root,
}

#[derive(Debug)]
pub struct AnalyzedUnit {
    pub unit_path: String,
    pub ir: RootIR,
    pub scope: ScopeID,
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

    let root = ctx.table.root();

    let entries = [
        ("bool", TypeDef::Bool),
        ("u8", TypeDef::Int(8, Signedness::Unsigned)),
        ("u16", TypeDef::Int(16, Signedness::Unsigned)),
        ("u32", TypeDef::Int(32, Signedness::Unsigned)),
        ("u64", TypeDef::Int(64, Signedness::Unsigned)),
        ("i8", TypeDef::Int(8, Signedness::Signed)),
        ("i16", TypeDef::Int(16, Signedness::Signed)),
        ("i32", TypeDef::Int(32, Signedness::Signed)),
        ("i64", TypeDef::Int(64, Signedness::Signed)),
        ("f32", TypeDef::Float(32)),
        ("f64", TypeDef::Float(64)),
        ("c_void", TypeDef::Empty),
    ];

    let mut ids = Vec::new();

    for (name, ty) in entries {
        let typeid = ctx.interner.intern(ty);

        let def = resolver::symbols::Definition {
            name: name.to_string(),
            mangled_name: None,
            kind: DefKind::TypeAlias(TypeAliasDef {
                resolved: true,
                typeid,
            }),
        };

        ctx.table.define(root, def).unwrap();
        ids.push(typeid);
    }

    let nothing = ctx.interner.intern(TypeDef::Empty);

    ctx.set_primitives(Primitives {
        nothing,
        bool_: ids[0],
        u8_: ids[1],
        u16_: ids[2],
        u32_: ids[3],
        u64_: ids[4],
        i8_: ids[5],
        i16_: ids[6],
        i32_: ids[7],
        i64_: ids[8],
        f32_: ids[9],
        f64_: ids[10],
    });
}
