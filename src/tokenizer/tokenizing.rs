use crate::tokenizer::{LiteralKind, Token, TokenKind};

use super::Cursor;
use super::{comment, identifier, keyword, literals, operator, whitespace};

pub struct Tokenizer<'a> {
    cursor: Cursor<'a>,
    text: &'a str,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = match self.cursor.first() {
            Some('1'..='9') => self.number_token(),
            Some('0') => match self.cursor.second() {
                Some('b' | 'B') => self.binary_token(),
                Some('x' | 'X') => self.hex_token(),
                _ => self.number_token(),
            },
            Some('"') => self.string_token(),
            Some('r') if self.cursor.second() == Some('#') => self.string_token(),
            Some('\u{0020}' | '\u{0009}' | '\u{000C}' | '\u{000A}' | '\u{000D}') => {
                self.whitespace_token()
            }
            Some('/') => match self.cursor.second() {
                Some('/') => self.comment_token(),
                _ => self.operator_token(),
            },
            Some('=' | '>' | '<' | '!' | '~' | '+' | '-' | '*' | '&' | '|' | '^' | '%') => {
                self.operator_token()
            }
            Some('(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.' | ':') => {
                self.separator_token()
            }
            Some(id) if identifier::is_identifier_start(id) => self.identifier_related_token(),
            Some(unexpected) => unreachable!("Unexpected char reached: {}", unexpected),
            None => return None,
        };
        Some(token)
    }
}

impl<'a> Tokenizer<'a> {
    pub fn tokenize(text: &'a str) -> Self {
        let cursor = Cursor::from_iter(text.chars());
        Tokenizer { cursor, text }
    }

    fn number_token(&mut self) -> Token<'a> {
        self.literal_token(literals::number::number)
    }

    fn binary_token(&mut self) -> Token<'a> {
        self.literal_token(literals::number::binary)
    }

    fn hex_token(&mut self) -> Token<'a> {
        self.literal_token(literals::number::hex)
    }

    fn string_token(&mut self) -> Token<'a> {
        self.literal_token(literals::string::string)
    }

    fn literal_token<F>(&mut self, target_fn: F) -> Token<'a>
    where
        F: Fn(&mut Cursor<'a>) -> (usize, LiteralKind),
    {
        let (size, kind) = target_fn(&mut self.cursor);
        let lexeme = self.eat_chars(size);
        Token {
            kind: TokenKind::Literal(kind),
            lexeme,
        }
    }

    fn comment_token(&mut self) -> Token<'a> {
        self.non_literal_token(comment::comment, TokenKind::Comment)
    }

    fn whitespace_token(&mut self) -> Token<'a> {
        self.non_literal_token(whitespace::whitespace, TokenKind::WhiteSpace)
    }

    fn operator_token(&mut self) -> Token<'a> {
        self.non_literal_token(operator::operator, TokenKind::Operator)
    }

    // separators: "(){}[];,:."
    fn separator_token(&mut self) -> Token<'a> {
        let _ = self.cursor.bump().expect("Checked in match statement");
        let lexeme = self.eat_chars(1);
        Token {
            kind: TokenKind::Separator,
            lexeme,
        }
    }

    fn non_literal_token<F>(&mut self, tokenizing_fn: F, kind: TokenKind) -> Token<'a>
    where
        F: Fn(&mut Cursor<'a>) -> usize,
    {
        let size = tokenizing_fn(&mut self.cursor);
        let lexeme = self.eat_chars(size);
        Token { kind, lexeme }
    }

    fn identifier_related_token(&mut self) -> Token<'a> {
        let size = identifier::identifier(&mut self.cursor);
        let lexeme = self.eat_chars(size);
        let kind = if keyword::is_keyword(lexeme) {
            TokenKind::Keyword
        } else if literals::boolean::is_boolean(lexeme) {
            TokenKind::Literal(LiteralKind::Boolean)
        } else {
            TokenKind::Identifier
        };
        Token { kind, lexeme }
    }

    fn eat_chars(&mut self, length: usize) -> &'a str {
        let (lexeme, remaining) = self.text.split_at(length);
        self.text = remaining;
        lexeme
    }
}

#[cfg(test)]
mod token_iter_tests {
    use super::Tokenizer;
    use super::{LiteralKind::*, Token, TokenKind::*};
    #[test]
    fn test_tokenizing() {
        let source = r#"test true 0.3 3 0xAB 0b10 struct;variable.function()+"string"+"\n""#;
        let tokens = Tokenizer::tokenize(source).collect::<Vec<_>>();
        let expected = vec![
            Token {
                kind: Identifier,
                lexeme: "test",
            },
            Token {
                kind: WhiteSpace,
                lexeme: " ",
            },
            Token {
                kind: Literal(Boolean),
                lexeme: "true",
            },
            Token {
                kind: WhiteSpace,
                lexeme: " ",
            },
            Token {
                kind: Literal(Floating),
                lexeme: "0.3",
            },
            Token {
                kind: WhiteSpace,
                lexeme: " ",
            },
            Token {
                kind: Literal(Integer),
                lexeme: "3",
            },
            Token {
                kind: WhiteSpace,
                lexeme: " ",
            },
            Token {
                kind: Literal(Hex),
                lexeme: "0xAB",
            },
            Token {
                kind: WhiteSpace,
                lexeme: " ",
            },
            Token {
                kind: Literal(Binary),
                lexeme: "0b10",
            },
            Token {
                kind: WhiteSpace,
                lexeme: " ",
            },
            Token {
                kind: Keyword,
                lexeme: "struct",
            },
            Token {
                kind: Separator,
                lexeme: ";",
            },
            Token {
                kind: Identifier,
                lexeme: "variable",
            },
            Token {
                kind: Separator,
                lexeme: ".",
            },
            Token {
                kind: Identifier,
                lexeme: "function",
            },
            Token {
                kind: Separator,
                lexeme: "(",
            },
            Token {
                kind: Separator,
                lexeme: ")",
            },
            Token {
                kind: Operator,
                lexeme: "+",
            },
            Token {
                kind: Literal(String),
                lexeme: "\"string\"",
            },
            Token {
                kind: Operator,
                lexeme: "+",
            },
            Token {
                kind: Literal(String),
                lexeme: "\"\\n\"",
            },
        ];
        assert_eq!(expected, tokens);
    }
}
