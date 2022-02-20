const N: usize = include!(concat!(env!("OUT_DIR"), "/stdlib_count.rs"));

pub fn content() -> [&'static str; N] {
    include!(concat!(env!("OUT_DIR"), "/stdlib.rs"))
}

pub fn paths() -> [&'static str; N] {
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
    paths()
        .map(|path| {
            path.rsplit_once(std::path::MAIN_SEPARATOR)
                .unwrap_or(("", ""))
                .0
        })
        .map(|path| path.split(std::path::MAIN_SEPARATOR).collect())
}
