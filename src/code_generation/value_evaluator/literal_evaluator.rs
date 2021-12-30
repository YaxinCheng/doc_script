use super::string_evaluator;
use super::value::Value;
use crate::tokenizer::LiteralKind;

pub fn evaluate<'ast, 'a>(kind: &LiteralKind, lexeme: &'a str) -> Value<'ast, 'a> {
    match kind {
        LiteralKind::Integer => Value::Int(lexeme.parse().expect("integer")),
        LiteralKind::Binary => Value::Int(isize::from_str_radix(&lexeme[2..], 2).expect("binary")),
        LiteralKind::Hex => Value::Int(isize::from_str_radix(&lexeme[2..], 16).expect("hex")),
        LiteralKind::Boolean => Value::Bool(lexeme.parse().expect("bool")),
        LiteralKind::Floating => Value::Float(lexeme.parse().expect("float")),
        LiteralKind::String => Value::String(string_evaluator::evaluate(lexeme)),
    }
}
