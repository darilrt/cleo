use std::io::Write;
use std::{env::current_dir, path::Path};

use errors::Error;
use project::Project;

const CLEO_HEADER_NAME: &str = "cleotypes.h";

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

            writeln!(buffer, "#include \"{}\"", CLEO_HEADER_NAME)?;
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

    generate_cleo_types(&output_dir)?;
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

fn generate_cleo_types(output_dir: &Path) -> Result<(), Error> {
    let content = "
#ifndef UINT32_MAX
typedef __UINT8_TYPE__     uint8_t;
typedef __INT8_TYPE__      int8_t;
typedef __UINT16_TYPE__    uint16_t;
typedef __INT16_TYPE__     int16_t;
typedef __UINT32_TYPE__    uint32_t;
typedef __INT32_TYPE__     int32_t;
typedef __UINT64_TYPE__    uint64_t;
typedef __INT64_TYPE__     int64_t;
#endif

#if !defined(__size_t_defined) && !defined(_SIZE_T_DEFINED) && !defined(_SIZE_T)
typedef __SIZE_TYPE__      size_t;
#define __size_t_defined
#define _SIZE_T_DEFINED
#define _SIZE_T
#endif

#if !defined(__ptrdiff_t_defined) && !defined(_PTRDIFF_T_DEFINED) && !defined(_PTRDIFF_T)
typedef __PTRDIFF_TYPE__   ptrdiff_t;
#define __ptrdiff_t_defined
#define _PTRDIFF_T_DEFINED
#define _PTRDIFF_T
#endif
";

    std::fs::write(&output_dir.join(CLEO_HEADER_NAME), content)?;

    Ok(())
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
    script_content.push_str("    exit 1\n"); // <- ESTA ES LA MAGIA
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
