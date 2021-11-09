use super::*;
use crate::ast::{Expression, Import, Name};
use crate::parser::Node;

#[test]
fn test_struct_content_newline_normal() {
    test_struct_init(
        r#"
const value = Test {
    42
    "hello"
}
    "#,
    )
}

#[test]
fn test_struct_content_without_start_newline() {
    test_struct_init(
        r#"
    const value = Test { 42
    "hello"
    }
    "#,
    )
}

#[test]
fn test_struct_content_without_end_newline() {
    test_struct_init(
        r#"
    const value = Test {
    42
    "hello"}
    "#,
    )
}

#[test]
fn test_struct_content_without_both_newline() {
    test_struct_init(
        r#"
    const value = Test {   42
    "hello"}
    "#,
    )
}

#[test]
#[should_panic]
fn test_struct_without_newline() {
    test_struct_init(r#"const value = Test { 42 "hello" }"#)
}

fn test_struct_init(text: &str) {
    let parse_tree = parse(tokenize(text));
    let expression = depth_first_find(parse_tree.root, NodeKind::Expression)
        .map(Expression::from)
        .expect("Failed to find expression");
    let expected = Expression::StructInit {
        name: Name::Simple("Test"),
        parameters: vec![],
        body: vec![
            Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "42",
            },
            Expression::Literal {
                kind: LiteralKind::String,
                lexeme: r#""hello""#,
            },
        ],
    };
    assert_eq!(expression, expected)
}

#[test]
fn test_import_statement_normal() {
    test_import_statement(
        r#"
    use test.import.{ first, second }
    "#,
    )
}

#[test]
fn test_import_statement_with_starting_new_line() {
    test_import_statement(
        r#"
    use test.import.{
        first, second }
    "#,
    )
}

#[test]
fn test_import_statement_with_new_line_in_between() {
    test_import_statement(
        r#"
    use test.import.{ first, 
    second }
    "#,
    )
}

#[test]
fn test_import_statement_with_comma_newline() {
    test_import_statement(
        r#"
    use test.import. { first,
    second,
    }
    "#,
    )
}

fn test_import_statement(text: &str) {
    let parse_tree = parse(tokenize(text));
    let import = depth_first_find(
        parse_tree.root,
        NodeKind::MultipleImportDeclarationStatement,
    )
    .map(Import::from)
    .expect("Failed to find import");
    let expected = Import::Multiple {
        prefix: Name::Qualified(vec!["test", "import"]),
        suffices: vec![Name::Simple("first"), Name::Simple("second")],
    };
    assert_eq!(import, expected)
}

fn depth_first_find(start_node: Node, node_kind: NodeKind) -> Option<Node> {
    DepthFirst::find(
        start_node,
        |node| node.kind() == Some(node_kind),
        |node| node.children().unwrap_or_default(),
    )
    .next()
}
