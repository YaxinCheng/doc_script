use super::LiteralKind;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Comment,
    WhiteSpace,
    NewLine,
    Identifier,
    Operator,
    Separator,
    Keyword,
    Literal(LiteralKind),

    // Reserved for parser
    ParsingStart,
    ParsingEnd,
}

impl<'a> Token<'a> {
    pub(in crate::tokenizer) fn should_keep(&self) -> bool {
        !matches!(self.kind, TokenKind::WhiteSpace | TokenKind::Comment)
    }

    /// Returns true if the current token suppresses following new line tokens
    ///
    /// It is normally used to skip formatting related new lines. For example
    /// ```doc_script
    /// use std::{name1,
    ///  name2}
    /// ```
    /// The new line above does not make `name1` a separate statement,
    /// it is there only for the purpose of formatting or readability.
    /// Therefore, it is better to suppress and skip it.
    pub fn suppress_new_line(&self) -> bool {
        matches!(
            self,
            Token {
                kind: TokenKind::Separator,
                lexeme: "{" | "," | "." | ";"
            } | Token {
                kind: TokenKind::Operator,
                lexeme: "="
            } | Token {
                kind: TokenKind::NewLine,
                lexeme: _
            }
        )
    }
}
