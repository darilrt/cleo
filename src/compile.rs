use std::io::Write;
use std::{env::current_dir, path::Path};

use errors::Error;
use project::Project;

pub fn compile(entry: &Path) -> Result<(), Error> {
    let workdir = current_dir().expect("Failed to get current working directory");

    let project = Project::load_project(entry).expect("Failed to load project");

    let result = project.analyze().expect("Failed to load project");

    let output_dir = create_output_dir(&workdir);
    let codegen = codegen::Codegen::new(&result.ctx);

    for unit in result.units {
        let output_file = output_dir.join(unit.unit_path.replace(".", "/"));

        std::fs::create_dir_all(output_file.parent().unwrap())
            .expect("Failed to create output directory");

        {
            let header = output_file.with_extension("h");
            let mut buffer: Vec<u8> = Vec::new();
            writeln!(buffer, "#include <stdint.h>")?;
            writeln!(buffer, "#include <stdio.h>\n")?;
            writeln!(buffer, "#include <stdlib.h>\n")?;
            codegen.emit_header(&mut buffer, unit.scope)?;
            std::fs::write(&header, buffer).expect("Failed to write output file");
        }

        {
            let source = output_file.with_extension("c");
            let mut buffer: Vec<u8> = Vec::new();
            for fnir in unit.ir.fns() {
                codegen.emit_fn_def(&mut buffer, unit.scope, fnir)?;
            }
            std::fs::write(&source, buffer).expect("Failed to write output file");
        }
    }

    Ok(())
}

fn create_output_dir(base_path: &Path) -> std::path::PathBuf {
    let output_dir = base_path.join("out");
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    }
    output_dir
}
