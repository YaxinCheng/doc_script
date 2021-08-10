pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str
}

#[derive(Copy, Clone, Debug)]
pub enum TokenKind {
    Comment,
    WhiteSpace,
    Identifier,
    Operator,
    Separator,
    Keyword,
    Literal(LiteralKind)
}

#[derive(Copy, Clone, Debug)]
pub enum LiteralKind {
    String,
    Integer,
    Floating,
    Boolean,
    Binary
}