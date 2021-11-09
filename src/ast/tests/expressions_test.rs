use super::super::{Expression, Name, Parameter};
use super::*;
use crate::search::BreadthFirst;

#[test]
fn test_struct_init_simple() {
    test_struct_init_basic("const view = View()\n", vec![], vec![])
}

#[test]
fn test_struct_init_empty_body() {
    test_struct_init_basic("const view = View() { }\n", vec![], vec![])
}

#[test]
fn test_struct_init_eliminate_parameters() {
    test_struct_init_basic("const view = View { }\n", vec![], vec![])
}

#[test]
#[should_panic]
fn test_struct_init_just_name() {
    test_struct_init_basic("const view = View\n", vec![], vec![])
}

#[test]
fn test_struct_with_single_parameter() {
    test_struct_init_basic(
        "const view = View(3)\n",
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
        "const view = View(3, \"string\", 3.14, false)\n",
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
        "const view = View(background_colour: \"red\", width:30, )\n",
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
        "const view = View(\"red\", width: 30)\n",
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
        "const view = View { Text(\"label\")\n View()\n }\n",
        vec![],
        vec![
            Expression::MethodInvocation {
                name: Name::Simple("Text"),
                parameters: vec![Parameter::Plain(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: "\"label\"",
                })],
            },
            Expression::MethodInvocation {
                name: Name::Simple("View"),
                parameters: vec![],
            },
        ],
    );
}

#[test]
fn test_struct_init_nested_body() {
    test_struct_init_basic(
        "const view = View { Text(\"label\")\n View() { Text(\"nested\") }\n}\n",
        vec![],
        vec![
            Expression::MethodInvocation {
                name: Name::Simple("Text"),
                parameters: vec![Parameter::Plain(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: "\"label\"",
                })],
            },
            Expression::StructInit {
                name: Name::Simple("View"),
                parameters: vec![],
                body: vec![Expression::MethodInvocation {
                    name: Name::Simple("Text"),
                    parameters: vec![Parameter::Plain(Expression::Literal {
                        kind: LiteralKind::String,
                        lexeme: "\"nested\"",
                    })],
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
        |node| {
            matches!(
                node.kind(),
                Some(NodeKind::StructInitExpression | NodeKind::MethodInvocation)
            )
        },
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find StructInitExpression or MethodInvocation");
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
        Expression::MethodInvocation { name, parameters } => {
            assert_eq!(name, Name::Simple("View"));
            assert_eq!(parameters, expected_parameters);
        }
        expression => panic!("Unexpected expression: {:?}", expression),
    }
}

#[test]
fn test_method_invocation_one_line() {
    test_method_invocation("const depth = 3.pow(2).abs()\n")
}

#[test]
fn test_chaining_method_invocation_multi_lines() {
    test_method_invocation(
        r#"const depth = 3
    .pow(
    2
    )
        .abs()
    "#,
    )
}

fn test_method_invocation(statement: &str) {
    let node = parse(tokenize(statement)).root;
    let expression = BreadthFirst::find(
        node,
        |node| matches!(node.kind(), Some(NodeKind::Expression)),
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .map(Expression::from)
    .expect("Cannot find Expression node");
    assert_eq!(
        expression,
        Expression::ChainingMethodInvocation {
            receiver: Box::new(Expression::ChainingMethodInvocation {
                receiver: Box::new(Expression::Literal {
                    kind: LiteralKind::Integer,
                    lexeme: "3",
                }),
                name: "pow",
                parameters: vec![Parameter::Plain(Expression::Literal {
                    kind: LiteralKind::Integer,
                    lexeme: "2",
                })],
            }),
            name: "abs",
            parameters: vec![],
        }
    )
}
