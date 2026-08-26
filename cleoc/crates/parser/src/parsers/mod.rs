use chumsky::{
    Parser,
    input::Input,
    span::{SimpleSpan, Span},
};

use crate::{
    errors::{self, Kind},
    parsers::unit::{Unit, unit},
};

mod block;
mod decl;
mod expr;
mod fn_decl;
mod generic_params;
mod ident;
mod import;
mod local;
mod path;
mod ptype;
mod stmt;
mod type_decl;
mod unit;

pub mod ast {
    pub use super::block::Block;
    pub use super::decl::Decl;
    pub use super::expr::{
        Expr, ExprAccess, ExprAssign, ExprCall, ExprIf, ExprInit, ExprInitField, ExprValue,
        Operator,
    };
    pub use super::fn_decl::{FnDecl, FnParam, FnSignature};
    pub use super::generic_params::{GenericParam, GenericParams};
    pub use super::ident::Ident;
    pub use super::import::ImportDecl;
    pub use super::local::{Binding, Local};
    pub use super::path::{PathExpr, Segment};
    pub use super::ptype::Type;
    pub use super::stmt::Stmt;
    pub use super::type_decl::{EnumValue, StructField, TraitMethod, TypeBody, TypeDecl};
    pub use super::unit::Unit;
}

pub fn parse<'a>(source: &'a str) -> errors::Result<'a, Unit> {
    use lexer::lex;

    let lexed = lex(source)?;

    let stream = lexed.iter().map(|t| {
        (
            t.kind.clone(),
            SimpleSpan::new((), t.span.start..t.span.end),
        )
    });

    let stream = chumsky::input::Stream::from_iter(stream)
        .map((0..source.len()).into(), |(t, s): (_, _)| (t, s));

    let result = unit().parse(stream);

    if result.has_errors() {
        let err = result
            .errors()
            .map(|e| e.clone().into_owned())
            .collect::<Vec<_>>();
        return Err(Kind::ParseError(err));
    }

    Ok(result.output().unwrap().to_owned())
}

mod test {
    #[test]
    fn test_parser() {
        use crate::unwrap_or_report;

        use super::parse;

        let source = r#"fn foo[T]() {
    a
    print("Hello, World")
}"#;
        let result = parse(source);

        unwrap_or_report!(result, source);
    }

    #[test]
    fn test_self_path_in_fn() {
        use super::parse;
        use crate::unwrap_or_report;

        // Same-line block with no ASI semicolons; `self` as param and in path
        let source = "fn draw(self: *Shape) { var x = self.a }";
        let result = parse(source);

        unwrap_or_report!(result, source);
    }
}
