use parser::unwrap_or_report_file;
use resolver::{
    check,
    context::{Context, Primitives},
    defkinds::TypeAliasDef,
    resolve,
    symbols::DefKind,
};

fn load_primitives(ctx: &mut Context) {
    use types::defs::{Signedness, TypeDef};

    let root = ctx.table.root();

    let entries = [
        ("void", TypeDef::Void),
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

    ctx.set_primitives(Primitives {
        void: ids[0],
        bool_: ids[1],
        u8_: ids[2],
        u16_: ids[3],
        u32_: ids[4],
        u64_: ids[5],
        i8_: ids[6],
        i16_: ids[7],
        i32_: ids[8],
        i64_: ids[9],
        f32_: ids[10],
        f64_: ids[11],
    });
}
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <source-file>", args[0]);
        std::process::exit(1);
    }

    let source_file = &args[1];
    let source = std::fs::read_to_string(source_file).expect("Failed to read source file");
    let unit = unwrap_or_report_file!(parser::parse(&source), source_file, &source);

    let mut ctx = Context::new(Primitives::default());

    load_primitives(&mut ctx);

    let root = ctx.table.root();

    match resolve(&mut ctx, root, &unit) {
        Err(err) => {
            println!("Resolve error: {:?}", err);
            return;
        }
        Ok(_) => {}
    }

    let unit = check(&mut ctx, root, unit).expect("Check error: ");
    // println!("{:?}", unit);

    let mut output = Vec::new();
    if let Err(err) = codegen::generate(&mut ctx, &mut output, root, &unit) {
        println!("Codegen error: {:?}", err);
        return;
    }
    let s = std::str::from_utf8(&output).unwrap();
    println!("{}", s);

    // println!("{}", ctx.debug(types::ScopeID(0), false));
}
