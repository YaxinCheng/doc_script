pub use literals::LiteralKind;
pub use token::{Token, TokenKind};

mod comment;
mod cursor;
mod identifier;
mod keyword;
mod literals;
mod operator;
mod token;
mod tokenizing;
mod whitespace;

pub type Cursor<'a> = cursor::Cursor<std::str::Chars<'a>>;

pub fn tokenize(text: &str) -> impl Iterator<Item = Token> {
    tokenizing::Tokenizer::tokenize(text).filter(Token::should_keep)
}

pub use whitespace::is_whitespace_or_newline;
