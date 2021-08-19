pub use literals::LiteralKind;
pub use token::{Token, TokenKind};

mod comment;
mod cursor;
mod identifier;
mod keyword;
mod literals;
mod operator;
mod token;
mod tokenizer;
mod whitespace;

type Cursor<'a> = cursor::Cursor<std::str::Chars<'a>>;

pub fn tokenize(text: &str) -> impl Iterator<Item = Token> {
    tokenizer::Tokenizer::tokenize(text)
}
