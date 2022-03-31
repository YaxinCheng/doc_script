const N: usize = include!(concat!(env!("OUT_DIR"), "/stdlib_count.rs"));

pub const fn content() -> [&'static str; N] {
    include!(concat!(env!("OUT_DIR"), "/stdlib.rs"))
}

pub const fn paths() -> [&'static str; N] {
    include!(concat!(env!("OUT_DIR"), "/stdlib_path.rs"))
}

#[cfg(test)]
pub fn compiled_content<'a>() -> [crate::ast::AbstractSyntaxTree<'a>; N] {
    content()
        .map(crate::tokenizer::tokenize)
        .map(crate::parser::parse)
        .map(crate::ast::abstract_tree)
}

#[cfg(test)]
pub fn module_paths() -> [Vec<&'static str>; N] {
    paths().map(std::path::Path::new).map(|path| {
        let mut components = path
            .components()
            .map(|component| component.as_os_str())
            .map(|name| name.to_str().expect("Not Utf-8"))
            .collect::<Vec<_>>();
        components.pop();
        components
    })
}
