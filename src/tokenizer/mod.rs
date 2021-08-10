mod comment;
mod cursor;
mod token;
mod whitespace;
mod identifier;

type Cursor<'a> = cursor::Cursor<std::str::Chars<'a>>;
pub use token::{Token, TokenKind, LiteralKind};

