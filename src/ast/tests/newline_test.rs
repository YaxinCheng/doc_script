use super::*;
use crate::ast::{ConstantDeclaration, Expression, Import, Name, Statement};
use crate::parser::Node;
use crate::search::BreadthFirst;

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
    test_struct_init(
        r#"const value = Test { 42 "hello" }
    "#,
    )
}

#[test]
fn test_struct_one_liners() {
    let parse_tree = parse(tokenize("const value = Test { 42 }\n"));
    let expression = breadth_first_find(parse_tree.root, NodeKind::Expression)
        .map(Expression::from)
        .next()
        .expect("Failed to find expression");
    let expected = Expression::StructInit {
        name: Name::simple("Test"),
        parameters: vec![],
        init_content: Some(
            vec![Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "42",
            }]
            .into(),
        ),
    };
    assert_eq!(expression, expected)
}

fn test_struct_init(text: &str) {
    let parse_tree = parse(tokenize(text));
    let expression = breadth_first_find(parse_tree.root, NodeKind::Expression)
        .map(Expression::from)
        .next()
        .expect("Failed to find expression");
    let expected = Expression::StructInit {
        name: Name::simple("Test"),
        parameters: vec![],
        init_content: Some(
            vec![
                Expression::Literal {
                    kind: LiteralKind::Integer,
                    lexeme: "42",
                },
                Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""hello""#,
                },
            ]
            .into(),
        ),
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
    let import = breadth_first_find(
        parse_tree.root,
        NodeKind::MultipleImportDeclarationStatement,
    )
    .map(Import::from)
    .next()
    .expect("Failed to find import");
    let expected = Import::Multiple {
        prefix: vec!["test", "import"],
        suffices: vec![vec!["first"], vec!["second"]],
    };
    assert_eq!(import, expected)
}

#[test]
fn test_semicolon_and_new_line_constants() {
    test_constants_separated(
        "
    const first = 1;

    const second = 2;
    ",
    )
}

#[test]
fn test_equal_sign_suppress_newline() {
    test_constants_separated(
        "
    const first = 
        1
    const second = 2
    ",
    )
}

fn test_constants_separated(program: &str) {
    let parse_tree = parse(tokenize(program));
    let imports: Vec<_> =
        breadth_first_find(parse_tree.root, NodeKind::ConstantDeclarationStatement)
            .map(Statement::from)
            .collect();
    let expected = vec![
        Statement::ConstantDeclaration(ConstantDeclaration {
            name: "first",
            value: Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "1",
            },
        }),
        Statement::ConstantDeclaration(ConstantDeclaration {
            name: "second",
            value: Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "2",
            },
        }),
    ];
    assert_eq!(imports, expected)
}

#[test]
fn test_block_one_liner() {
    let parse_tree = parse(tokenize("const block = { 42 }\n"));
    let expression = breadth_first_find(parse_tree.root, NodeKind::Expression)
        .map(Expression::from)
        .next()
        .expect("Failed to find expression");
    let expected = Expression::Block(
        vec![Statement::Expression(Expression::Literal {
            kind: LiteralKind::Integer,
            lexeme: "42",
        })]
        .into(),
    );
    assert_eq!(expression, expected)
}

#[test]
fn test_block_normal() {
    test_block(
        r#"const block = {
    42
    "hello world"
    }
    "#,
    )
}

#[test]
fn test_block_without_starting_new_line() {
    test_block(
        r#"const block = {42
    "hello world"
    }
    "#,
    )
}

#[test]
fn test_block_without_ending_new_line() {
    test_block(
        r#"const block = {
    42
    "hello world"}
    "#,
    )
}

#[test]
fn test_block_without_new_lines() {
    test_block(
        r#"const block = {42
    "hello world"}
    "#,
    )
}

#[test]
fn test_block_with_empty_new_lines() {
    test_block(
        r#"const block = 
    {
       
        42
    //comment
    "hello world"
        } 
    "#,
    )
}

fn test_block(program: &str) {
    let parse_tree = parse(tokenize(program));
    let expression = breadth_first_find(parse_tree.root, NodeKind::Expression)
        .map(Expression::from)
        .next()
        .expect("Failed to find expression");
    let expected = Expression::Block(
        vec![
            Statement::Expression(Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "42",
            }),
            Statement::Expression(Expression::Literal {
                kind: LiteralKind::String,
                lexeme: r#""hello world""#,
            }),
        ]
        .into(),
    );
    assert_eq!(expression, expected)
}

fn breadth_first_find(start_node: Node, node_kind: NodeKind) -> impl Iterator<Item = Node> {
    BreadthFirst::find(
        start_node,
        move |node| node.kind() == Some(node_kind),
        |node| node.children().unwrap_or_default(),
    )
}
