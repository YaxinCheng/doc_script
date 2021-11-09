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
}
