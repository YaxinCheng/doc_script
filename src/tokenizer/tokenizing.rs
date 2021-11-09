use crate::tokenizer::{LiteralKind, Token, TokenKind};

use super::Cursor;
use super::{comment, identifier, keyword, literals, operator, whitespace};

pub struct Tokenizer<'a> {
    cursor: Cursor<'a>,
    text: &'a str,
    unbalanced_brackets: Vec<char>,
    meaningful_content_count: usize,
    last_token: Option<Token<'a>>,
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
            Some('\u{0020}' | '\u{0009}' | '\u{000C}' | '\u{000A}' | '\u{000D}')
                if self.last_token_suppresses_newline() =>
            {
                self.whitespace_and_newline_token()
            }
            Some('\u{0020}' | '\u{0009}' | '\u{000C}') => self.whitespace_token(),
            Some('\u{000A}' | '\u{000D}') => self.newline_token(),
            Some('/') => match self.cursor.second() {
                Some('/') => self.comment_token(),
                _ => self.operator_token(),
            },
            Some('\\') => self.escape_token(),
            Some('=' | '>' | '<' | '!' | '~' | '+' | '-' | '*' | '&' | '|' | '^' | '%') => {
                self.operator_token()
            }
            Some(bracket @ ('(' | '[' | ')' | ']' | '{' | '}')) => {
                if bracket == '(' || bracket == '[' || bracket == '{' {
                    self.unbalanced_brackets.push(bracket)
                } else {
                    self.unbalanced_brackets.pop();
                }
                self.separator_token()
            }
            Some(';' | ',' | '.' | ':') => self.separator_token(),
            Some(id) if identifier::is_identifier_start(id) => self.identifier_related_token(),
            Some(unexpected) => unreachable!("Unexpected char reached: {}", unexpected),
            None => return None,
        };
        if !matches!(
            token.kind,
            TokenKind::WhiteSpace | TokenKind::Comment | TokenKind::NewLine
        ) {
            self.meaningful_content_count += 1;
        }
        self.last_token.replace(token);
        Some(token)
    }
}

impl<'a> Tokenizer<'a> {
    pub fn tokenize(text: &'a str) -> Self {
        let cursor = Cursor::from_iter(text.chars());
        Tokenizer {
            cursor,
            text,
            unbalanced_brackets: vec![],
            meaningful_content_count: 0,
            last_token: None,
        }
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

    fn newline_token(&mut self) -> Token<'a> {
        let allowed_in_brackets = matches!(&self.unbalanced_brackets.last(), None | Some('{'));
        let has_meaningful_content = self.meaningful_content_count > 0;
        let token = self.non_literal_token(whitespace::newline, TokenKind::NewLine);
        if allowed_in_brackets && has_meaningful_content {
            self.meaningful_content_count = 0;
            token
        } else {
            Token {
                kind: TokenKind::WhiteSpace,
                lexeme: token.lexeme,
            }
        }
    }

    fn whitespace_and_newline_token(&mut self) -> Token<'a> {
        self.non_literal_token(whitespace::whitespace_and_newline, TokenKind::WhiteSpace)
    }

    fn escape_token(&mut self) -> Token<'a> {
        let back_slash = self.cursor.bump().expect("Backslash not present");
        let newline_and_whitespaces_length = whitespace::whitespace_and_newline(&mut self.cursor);
        if newline_and_whitespaces_length == 0 {
            panic!("Only newline and whitespaces can be escaped")
        }
        let lexeme = self.eat_chars(back_slash.len_utf8() + newline_and_whitespaces_length);
        Token {
            kind: TokenKind::WhiteSpace,
            lexeme,
        }
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
        debug_assert!(length > 0, "Cannot eat lexeme with size 0");
        let (lexeme, remaining) = self.text.split_at(length);
        self.text = remaining;
        lexeme
    }

    fn last_token_suppresses_newline(&self) -> bool {
        matches!(
            self.last_token,
            Some(Token {
                kind: TokenKind::Separator,
                lexeme: "{" | ","
            })
        )
    }
}

#[cfg(test)]
mod token_iter_tests {
    use super::Tokenizer;
    use super::{LiteralKind::*, Token, TokenKind::*};
    use crate::tokenizer::tokenize;
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

    #[test]
    fn test_newline_inside_brackets() {
        let text = "(1+\n3)";
        let tokens = tokenize(text).collect::<Vec<_>>();
        let expected = vec![
            Token {
                kind: Separator,
                lexeme: "(",
            },
            Token {
                kind: Literal(Integer),
                lexeme: "1",
            },
            Token {
                kind: Operator,
                lexeme: "+",
            },
            Token {
                kind: Literal(Integer),
                lexeme: "3",
            },
            Token {
                kind: Separator,
                lexeme: ")",
            },
        ];
        assert_eq!(tokens, expected)
    }

    #[test]
    fn test_newline_outside_of_brackets() {
        let text = "const value = 3\n";
        test_newline_with_or_without_escape(text)
    }

    #[test]
    fn test_escape_newline() {
        let text = r#"const value\
          = 3
"#;
        test_newline_with_or_without_escape(text)
    }

    #[test]
    fn test_meaningless_newlines() {
        let text = r#"
        
// this is a comment
const value\
\
= 3
"#;
        test_newline_with_or_without_escape(text)
    }

    fn test_newline_with_or_without_escape(text: &str) {
        let tokens = tokenize(text).collect::<Vec<_>>();
        let expected = vec![
            Token {
                kind: Keyword,
                lexeme: "const",
            },
            Token {
                kind: Identifier,
                lexeme: "value",
            },
            Token {
                kind: Operator,
                lexeme: "=",
            },
            Token {
                kind: Literal(Integer),
                lexeme: "3",
            },
            Token {
                kind: NewLine,
                lexeme: "\n",
            },
        ];
        assert_eq!(expected, tokens)
    }

    #[test]
    fn test_newline_in_nested_brackets() {
        let text = "func(View {\n3\n5\n }\n)";
        let tokens: Vec<_> = tokenize(text).collect();
        let expected = vec![
            Token {
                kind: Identifier,
                lexeme: "func",
            },
            Token {
                kind: Separator,
                lexeme: "(",
            },
            Token {
                kind: Identifier,
                lexeme: "View",
            },
            Token {
                kind: Separator,
                lexeme: "{",
            },
            Token {
                kind: Literal(Integer),
                lexeme: "3",
            },
            Token {
                kind: NewLine,
                lexeme: "\n",
            },
            Token {
                kind: Literal(Integer),
                lexeme: "5",
            },
            Token {
                kind: NewLine,
                lexeme: "\n",
            },
            Token {
                kind: Separator,
                lexeme: "}",
            },
            Token {
                kind: Separator,
                lexeme: ")",
            },
        ];
        assert_eq!(tokens, expected)
    }
}
