use crate::tokenizer::Token;

mod tokenizer;

pub fn tokenize(text: &str) -> impl Iterator<Item = Token> {
    tokenizer::tokenize(text)
}
