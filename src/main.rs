mod compile;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <source-file>", args[0]);
        std::process::exit(1);
    }

    let path = std::path::Path::new(&args[1]);
    compile::compile(path);
}

fn _print_side_by_side(src: &str, result: &str) {
    let src_lines: Vec<&str> = src.lines().collect();
    let result_lines: Vec<&str> = result.lines().collect();

    let max_lines = src_lines.len().max(result_lines.len());
    let line_number_width = max_lines.to_string().len();

    // Ancho fijo para la columna izquierda (ajustable)
    let left_width = 60;

    // Header
    println!(
        "{:>width$} | {:<left_width$} | {}",
        "",
        "SRC",
        "RESULT",
        width = line_number_width,
        left_width = left_width
    );
    println!(
        "{:-<width$}-+-{:-<left_width$}-+-{:-<40}",
        "",
        "",
        "",
        width = line_number_width,
        left_width = left_width
    );

    for i in 0..max_lines {
        let left = src_lines.get(i).unwrap_or(&"");
        let right = result_lines.get(i).unwrap_or(&"");

        // Truncamos líneas muy largas para que no se rompa el formato
        let left_display = if left.len() > left_width {
            format!("{}…", &left[..left_width - 1])
        } else {
            left.to_string()
        };

        println!(
            "{:>width$} | {:<left_width$} | {}",
            i + 1,
            left_display,
            right,
            width = line_number_width,
            left_width = left_width
        );
    }
}
