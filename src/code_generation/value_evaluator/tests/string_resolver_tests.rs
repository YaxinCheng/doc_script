use super::super::string_evaluator::evaluate;
use std::borrow::Cow;

#[test]
fn test_simple_string() {
    let actual = evaluate(r#""hello world""#);
    let expected = "hello world";
    assert!(matches!(actual, Cow::Borrowed(_)));
    assert_eq!(actual, expected)
}

#[test]
fn test_simple_string_with_escape() {
    let actual = evaluate(r#""hello\nworld\t\"\'\\\r""#);
    let expected = "hello\nworld\t\"\'\\\r";
    assert_eq!(actual, expected)
}

#[test]
fn test_simple_string_escape_spaces() {
    let actual = evaluate(r#""hello \   world""#);
    let expected = "hello world";
    assert_eq!(actual, expected)
}

#[test]
fn test_simple_string_escape_newlines() {
    let actual = evaluate(
        r#""hello \
        world""#,
    );
    let expected = "hello world";
    assert_eq!(actual, expected)
}

#[test]
fn test_simple_string_multiple_lines() {
    let actual = evaluate(
        r#""multiple
        lines""#,
    );
    let expected = "multiple\n        lines";
    assert_eq!(actual, expected)
}

#[test]
fn test_raw_string() {
    let actual = evaluate(r##"r#"no escape like \n or space \  . It does nothing"#"##);
    let expected = "no escape like \\n or space \\  . It does nothing";
    assert_eq!(actual, expected)
}

#[test]
fn test_raw_string_multiple_lines() {
    let actual = evaluate(
        r##"r#"can still
        go multiple
        lines
        "#"##,
    );
    let expected = "can still\n        go multiple\n        lines\n        ";
    assert_eq!(actual, expected)
}
