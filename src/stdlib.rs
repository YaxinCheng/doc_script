include!(concat!(env!("OUT_DIR"), "/stdlib.rs"));

#[cfg(test)]
pub fn compiled_content<'a>() -> [crate::ast::AbstractSyntaxTree<'a>; N] {
    CONTENT
        .map(crate::tokenizer::tokenize)
        .map(crate::parser::parse)
        .map(crate::ast::abstract_tree)
}

#[cfg(test)]
pub fn module_paths() -> [Vec<&'static str>; N] {
    use std::path::{Component, Path};
    PATHS
        .map(Path::new)
        .map(|path| path.parent().unwrap_or(path))
        .map(Path::components)
        .map(|components| {
            components
                .map(Component::as_os_str)
                .map(|name| name.to_str().expect("Not Utf-8"))
                .collect()
        })
}
