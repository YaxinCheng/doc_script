use super::super::{Expression, Name, Parameter};
use super::*;

#[test]
fn test_constant_declaration() {
    let parse_tree = parse(tokenize(r#"const value = "String";"#));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::ConstantDeclarationExpression)),
        |node| node.children_owned().unwrap_or_default(),
    )
    .next()
    .expect("Couldn't find ConstantDeclarationExpression node");
    let expression = Expression::from(node);
    assert!(matches!(
        expression,
        Expression::Constant { name: "value", .. }
    ));
    match expression {
        Expression::Constant { name, data } => {
            assert_eq!(name, "value");
            assert!(matches!(
                *data,
                Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""String""#
                }
            ));
        }
        expression => panic!("Unexpected expression: {:?}", expression),
    }
}

#[test]
fn test_struct_init_simple() {
    test_struct_init_basic(r#"const view = View();"#, vec![], vec![])
}

#[test]
fn test_struct_init_empty_body() {
    test_struct_init_basic(r#"const view = View() { };"#, vec![], vec![])
}

#[test]
fn test_struct_init_eliminate_parameters() {
    test_struct_init_basic(r#"const view = View { };"#, vec![], vec![])
}

#[test]
#[should_panic]
fn test_struct_init_just_name() {
    test_struct_init_basic(r#"const view = View;"#, vec![], vec![])
}

#[test]
fn test_struct_with_single_parameter() {
    test_struct_init_basic(
        r#"const view = View(3);"#,
        vec![Parameter::Plain(Expression::Literal {
            kind: LiteralKind::Integer,
            lexeme: "3",
        })],
        vec![],
    )
}

#[test]
fn test_struct_with_multiple_parameter() {
    test_struct_init_basic(
        r#"const view = View(3, "string", 3.14, false);"#,
        vec![
            Parameter::Plain(Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "3",
            }),
            Parameter::Plain(Expression::Literal {
                kind: LiteralKind::String,
                lexeme: r#""string""#,
            }),
            Parameter::Plain(Expression::Literal {
                kind: LiteralKind::Floating,
                lexeme: "3.14",
            }),
            Parameter::Plain(Expression::Literal {
                kind: LiteralKind::Boolean,
                lexeme: "false",
            }),
        ],
        vec![],
    )
}

#[test]
fn test_struct_init_with_labelled_parameter() {
    test_struct_init_basic(
        r#"const view = View(background_colour: "red", width:30, );"#,
        vec![
            Parameter::Labelled {
                label: "background_colour",
                content: Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: "\"red\"",
                },
            },
            Parameter::Labelled {
                label: "width",
                content: Expression::Literal {
                    kind: LiteralKind::Integer,
                    lexeme: "30",
                },
            },
        ],
        vec![],
    )
}

#[test]
fn test_struct_init_mixed_parameter_types() {
    test_struct_init_basic(
        r#"const view = View("red", width: 30);"#,
        vec![
            Parameter::Plain(Expression::Literal {
                kind: LiteralKind::String,
                lexeme: r#""red""#,
            }),
            Parameter::Labelled {
                label: "width",
                content: Expression::Literal {
                    kind: LiteralKind::Integer,
                    lexeme: "30",
                },
            },
        ],
        vec![],
    )
}

#[test]
fn test_struct_init_with_body() {
    test_struct_init_basic(
        r#"const view = View { Text("label"), View(), };"#,
        vec![],
        vec![
            Expression::StructInit {
                name: Name::Simple("Text"),
                parameters: vec![Parameter::Plain(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: "\"label\"",
                })],
                body: vec![],
            },
            Expression::StructInit {
                name: Name::Simple("View"),
                parameters: vec![],
                body: vec![],
            },
        ],
    );
}

#[test]
fn test_struct_init_nested_body() {
    test_struct_init_basic(
        r#"const view = View { Text("label"), View() { Text("nested"), } };"#,
        vec![],
        vec![
            Expression::StructInit {
                name: Name::Simple("Text"),
                parameters: vec![Parameter::Plain(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: "\"label\"",
                })],
                body: vec![],
            },
            Expression::StructInit {
                name: Name::Simple("View"),
                parameters: vec![],
                body: vec![Expression::StructInit {
                    name: Name::Simple("Text"),
                    parameters: vec![Parameter::Plain(Expression::Literal {
                        kind: LiteralKind::String,
                        lexeme: "\"nested\"",
                    })],
                    body: vec![],
                }],
            },
        ],
    )
}

fn test_struct_init_basic(
    statement: &str,
    expected_parameters: Vec<Parameter>,
    expected_body: Vec<Expression>,
) {
    let parse_tree = parse(tokenize(statement));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::StructInitExpression)),
        |node| node.children_owned().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find StructInitExpression");
    let expression = Expression::from(node);
    match expression {
        Expression::StructInit {
            name,
            parameters,
            body,
        } => {
            assert_eq!(name, Name::Simple("View"));
            assert_eq!(parameters, expected_parameters);
            assert_eq!(body, expected_body);
        }
        expression => panic!("Unexpected expression: {:?}", expression),
    }
}
