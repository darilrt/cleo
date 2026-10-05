use ast::Root;
use errors::Error;

pub fn get_imports(ast: &Root) -> Result<Vec<String>, Error> {
    let mut imports = Vec::new();

    for decl in &ast.decls {
        match decl {
            ast::Decl::Import(decl) => {
                let import_path = decl
                    .path
                    .iter()
                    .map(|ident| ident.str())
                    .collect::<Vec<_>>()
                    .join("/");
                imports.push(import_path);
            }
            _ => {}
        }
    }

    Ok(imports)
}
