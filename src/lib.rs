use std::path::Path;

mod ast;
mod parser;
mod search;
mod tokenizer;

pub fn compile(source_file_names: Vec<String>) -> String {
    let file_content = source_file_names.iter().map(read_file).collect::<Vec<_>>();
    let compiled_syntax_trees = file_content
        .iter()
        .map(|content| content.as_ref())
        .map(tokenizer::tokenize)
        .map(parser::parse)
        .map(ast::abstract_tree)
        .zip(source_file_names.iter().map(|name| name.as_str()))
        .collect::<Vec<_>>();
    String::new()
}

fn read_file<P: AsRef<Path>>(path: P) -> impl AsRef<str> {
    let mut file_content = std::fs::read_to_string(&path).expect("Failed to read file");
    file_content.push('\n');
    file_content.into_boxed_str()
}
