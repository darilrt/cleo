use parser::unwrap_or_report;

fn main() {
    // load a file from first argument, parse it, and print the AST or errors
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <source-file>", args[0]);
        std::process::exit(1);
    }

    let source_file = &args[1];

    let source = std::fs::read_to_string(source_file).expect("Failed to read source file");

    let unit = unwrap_or_report!(parser::parse(&source), &source);

    println!("{:#?}", unit);
}
