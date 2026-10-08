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

    let mut generated_c_files = Vec::new();

    for unit in result.units {
        let output_file = output_dir.join(unit.unit_path.replace(".", "/"));

        std::fs::create_dir_all(output_file.parent().unwrap())
            .expect("Failed to create output directory");

        {
            let header = output_file.with_extension("h");
            let header_guard = unit.unit_path.replace(".", "_").to_uppercase();

            let mut buffer: Vec<u8> = Vec::new();

            writeln!(buffer, "#ifndef CLEO_{}", header_guard)?;
            writeln!(buffer, "#define CLEO_{}", header_guard)?;
            writeln!(buffer, "")?;

            // writeln!(buffer, "#include <stdint.h>")?;
            unit.imports.iter().for_each(|import| {
                let import_path = import.replace(".", "/");
                writeln!(buffer, "#include \"{}.h\"", import_path).unwrap();
            });

            writeln!(buffer, "")?;

            codegen.emit_header(&mut buffer, unit.scope)?;
            writeln!(buffer, "#endif // CLEO_{}", header_guard)?;

            std::fs::write(&header, buffer).expect("Failed to write output file");
        }

        {
            let source = output_file.with_extension("c");
            let mut buffer: Vec<u8> = Vec::new();

            writeln!(buffer, "#include <stdint.h>\n",)?;

            writeln!(
                buffer,
                "#include \"{}.h\"",
                unit.unit_path.replace(".", "/")
            )?;
            writeln!(buffer, "")?;

            for fnir in unit.ir.fns() {
                codegen.emit_fn_def(&mut buffer, unit.scope, fnir)?;
            }
            std::fs::write(&source, buffer).expect("Failed to write output file");

            generated_c_files.push(source.to_string_lossy().to_string());
        }
    }

    generate_build_script(&output_dir, &generated_c_files)?;

    Ok(())
}

fn create_output_dir(base_path: &Path) -> std::path::PathBuf {
    let output_dir = base_path.join("out");
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    }
    output_dir
}

// Temporal helper to build projects
fn generate_build_script(output_dir: &Path, c_files: &[String]) -> Result<(), Error> {
    let script_path = output_dir.join("build.sh");
    let mut script_content = String::new();

    script_content.push_str("#!/bin/bash\n\n");
    script_content.push_str("echo \"Compiling Cleo project...\"\n\n");

    // Comando GCC base
    script_content.push_str("gcc -I. ");

    // Agregamos todos los archivos .c separados por un espacio
    for file in c_files {
        script_content.push_str(file);
        script_content.push(' ');
    }

    // Archivo de salida final
    script_content.push_str("-o app\n\n");

    // Mensaje de éxito/error opcional para mejor DX
    script_content.push_str("if [ $? -eq 0 ]; then\n");
    script_content.push_str("    echo \"Build successful! Run with ./app\"\n");
    script_content.push_str("else\n");
    script_content.push_str("    echo \"Build failed.\"\n");
    script_content.push_str("fi\n");

    std::fs::write(&script_path, script_content)
        .map_err(|e| Error::Generic(format!("Failed to write build.sh: {}", e)))?;

    // Opcional: Le da permisos de ejecución automáticamente en sistemas Unix (Linux/Mac)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = std::fs::metadata(&script_path).map(|m| m.permissions()) {
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&script_path, perms);
        }
    }

    Ok(())
}
