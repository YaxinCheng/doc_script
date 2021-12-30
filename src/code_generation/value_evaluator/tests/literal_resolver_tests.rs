use super::super::literal_evaluator::evaluate;
use super::super::value::Value;
use crate::tokenizer::LiteralKind;
use quickcheck_macros::quickcheck;
use std::str::FromStr;

#[quickcheck]
fn test_resolve_integer_valid(num: isize) -> bool {
    let string = num.to_string();
    let actual = evaluate(&LiteralKind::Integer, &string);
    let expected = Value::Int(num);
    actual == expected
}

#[test]
#[should_panic]
fn test_resolve_integer_overflow() {
    let number = format!("{}1", isize::MAX);
    let _ = evaluate(&LiteralKind::Integer, &number);
}

#[test]
fn test_resolve_binary() {
    let actual = evaluate(&LiteralKind::Binary, "0b1010");
    let expected = Value::Int(10);
    assert_eq!(actual, expected)
}

#[test]
fn test_resolve_hex() {
    let actual = evaluate(&LiteralKind::Hex, "0xA0");
    let expected = Value::Int(160);
    assert_eq!(actual, expected)
}

#[quickcheck]
fn test_float(num: u16) -> bool {
    let string = format!("{num}.{num}", num = num);
    let actual = evaluate(&LiteralKind::Floating, &string);
    let expected = Value::Float(f32::from_str(&string).expect("Float failed to parse"));
    expected == actual
}

#[test]
fn test_boolean_true() {
    let actual = evaluate(&LiteralKind::Boolean, "true");
    let expected = Value::Bool(true);
    assert_eq!(actual, expected)
}

#[test]
fn test_boolean_false() {
    let actual = evaluate(&LiteralKind::Boolean, "false");
    let expected = Value::Bool(false);
    assert_eq!(actual, expected)
}
