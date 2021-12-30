use crate::tokenizer::{is_whitespace_or_newline, Cursor};
use std::borrow::Cow;

pub fn evaluate(literal: &str) -> Cow<str> {
    let is_raw = literal.starts_with("r#");
    let content = strip_surroundings(literal);
    if is_raw || !content.contains('\\') {
        Cow::Borrowed(content)
    } else {
        Cow::Owned(resolve_simple_literal(content))
    }
}

fn strip_surroundings(lexeme: &str) -> &str {
    let first_quote = lexeme.find('"').expect("First quote");
    let last_quote = lexeme.rfind('"').expect("Last quote");
    &lexeme[first_quote + 1..last_quote]
}

fn resolve_simple_literal(literal: &str) -> String {
    let mut resolved = String::with_capacity(literal.len());
    let mut cursor = Cursor::from_iter(literal.chars());
    while let Some(leading) = cursor.bump() {
        if leading == '\\' {
            if let Some(escaped) =
                resolve_escaped(cursor.bump().expect("At least one char after \\"))
            {
                resolved.push(escaped);
            } else {
                cursor.eat_while(is_whitespace_or_newline);
            }
        } else {
            resolved.push(leading)
        }
    }
    resolved
}

fn resolve_escaped(c: char) -> Option<char> {
    match c {
        't' => Some('\t'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '\\' => Some('\\'),
        _ => None,
    }
}
