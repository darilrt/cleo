use parser::unwrap_or_report_file;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <source-file>", args[0]);
        std::process::exit(1);
    }

    let source_file = &args[1];

    let source = std::fs::read_to_string(source_file).expect("Failed to read source file");

    let unit = unwrap_or_report_file!(parser::parse(&source), source_file, &source);

    // let table = collector::collect(&unit);

    println!("{:#?}", unit);
}
