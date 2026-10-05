use std::path::{Path, PathBuf};

use errors::Error;
use resolver::{
    context::{Context, Primitives},
    defkinds::TypeAliasDef,
    symbols::DefKind,
};
use types::ScopeID;

#[derive(Debug, Clone)]
pub struct UnitInfo {
    pub path: PathBuf,
    pub scope: ScopeID,
    pub ast: ast::Root,
}

pub struct Project {
    entry_file: PathBuf,
    project_dir: PathBuf,
    ctx: Context,
    units: Vec<UnitInfo>,
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

        let entry_file = abs_path;

        let mut ctx = Context::new(Primitives::default());
        load_primitives(&mut ctx);

        Ok(Self {
            entry_file,
            project_dir,
            ctx,
            units: Vec::new(),
        })
    }

    pub fn load_project(entry: &Path) -> Result<(), Error> {
        let mut project = Project::new(entry)?;

        let root = project.ctx.table.root();

        project.load_unit(&project.entry_file, root, &project.project_dir)?;

        println!("Loaded project units: {:?}", project.units);

        Ok(())
    }

    fn load_unit(&mut self, path: &Path, scope: ScopeID, project_dir: &Path) -> Result<(), Error> {
        Ok(())
    }
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
