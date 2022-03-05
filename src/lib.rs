extern crate core;

use std::path::Path;

mod ast;
mod code_generation;
mod env;
mod parser;
mod search;
mod stdlib;
#[cfg(test)]
mod tests;
mod tokenizer;

const OUTPUT_FILE: &str = "a.dc";

pub fn compile(source_file_names: Vec<String>) {
    compile_to(source_file_names, |env| {
        code_generation::generate_code(env, OUTPUT_FILE)
    })
}

fn compile_to<O, F: FnOnce(&env::Environment) -> O>(
    source_file_names: Vec<String>,
    output_fn: F,
) -> O {
    let file_content = source_file_names.iter().map(read_file).collect::<Vec<_>>();
    let mut compiled_syntax_trees = stdlib::content()
        .into_iter()
        .chain(file_content.iter().map(|content| content.as_ref()))
        .map(tokenizer::tokenize)
        .map(parser::parse)
        .map(ast::abstract_tree)
        .collect::<Vec<_>>();
    let environment = env::Environment::builder()
        .add_modules_from_paths(
            stdlib::paths().into_iter().chain(
                source_file_names
                    .iter()
                    .map(String::as_str)
                    .inspect(prohibit_std_injection),
            ),
        )
        .generate_scopes(&mut compiled_syntax_trees)
        .resolve_names(&compiled_syntax_trees)
        .validate(&compiled_syntax_trees)
        .build();
    output_fn(&environment)
}

fn read_file<P: AsRef<Path>>(path: P) -> impl AsRef<str> {
    let mut file_content = std::fs::read_to_string(&path).expect("Failed to read file");
    file_content.push('\n');
    file_content.into_boxed_str()
}

fn prohibit_std_injection<P: AsRef<Path>>(path: &P) {
    if path.as_ref().starts_with("std") {
        panic!("Injecting into std is prohibited!")
    }
}
