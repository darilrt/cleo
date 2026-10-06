mod compile;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <source-file>", args[0]);
        std::process::exit(1);
    }

    let path = std::path::Path::new(&args[1]);

    match compile::compile(path) {
        Ok(_) => println!("Compilation successful!"),
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            std::process::exit(1);
        }
    }
}
