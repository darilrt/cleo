use std::path::Path;

use ast::Root;
use errors::Error;
use parser::unwrap_or_report_file;
use resolver::{
    context::{Context, Primitives},
    defkinds::TypeAliasDef,
    imports::get_imports,
    symbols::DefKind,
};
use types::ScopeID;

pub fn compile(entry: &Path) {
    let workdir = entry.parent().expect("Failed to get parent directory");
    let output_dir = create_output_dir(workdir);

    let mut ctx = Context::new(Primitives::default());
    load_primitives(&mut ctx);

    let root = ctx.table.root();

    let units_tree =
        load_project_units(&mut ctx, entry, root, workdir).expect("Failed to load project");
}

fn create_output_dir(base_path: &Path) -> std::path::PathBuf {
    let output_dir = base_path.join("out");
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    }
    output_dir
}

#[derive(Debug)]
pub struct UnitInfo {
    pub ast: Root,
    pub scope: ScopeID,
    pub path: std::path::PathBuf,
    pub imports: Vec<UnitInfo>,
}

fn load_project_units(
    ctx: &mut Context,
    entry: &Path,
    scope: ScopeID,
    workdir: &Path,
) -> Result<UnitInfo, Error> {
    let ast = unit_from_file(entry)?;
    let imports = get_imports(&ast)?;

    let mut units = Vec::new();

    for import in imports {
        let import_path = workdir.join(import).with_extension("cleo");

        let scope = ctx.table.push(scope)?;
        let uinfo = load_project_units(ctx, &import_path, scope, workdir)?;

        units.push(uinfo);
    }

    Ok(UnitInfo {
        ast,
        scope,
        imports: units,
        path: entry.to_path_buf(),
    })
}

fn unit_from_file(path: &Path) -> Result<Root, Error> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    Ok(unwrap_or_report_file!(
        parser::parse(&source),
        path.to_str().unwrap(),
        &source
    ))
}
