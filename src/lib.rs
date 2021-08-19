use crate::parser::ParseTree;

mod ast;
mod parser;
mod search;
mod tokenizer;

pub fn compile(text: &str) -> ParseTree {
    let tokens = tokenizer::tokenize(text);
    parser::parse(tokens)
}
