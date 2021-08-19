#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LiteralKind {
    String,
    Integer,
    Binary,
    Hex,
    Floating,
    Boolean,
}
