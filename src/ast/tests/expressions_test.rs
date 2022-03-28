use super::super::{Expression, Name};
use super::*;
use crate::ast::parameter::Parameter;
use crate::ast::{Accessor, StructInitContent};
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
#[should_panic]
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
            Expression::StructInit {
                name: Name::simple("Text"),
                parameters: vec![Parameter::Plain(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: "\"label\"",
                })],
                init_content: None,
            },
            Expression::StructInit {
                name: Name::simple("View"),
                parameters: vec![],
                init_content: None,
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
            Expression::StructInit {
                name: Name::simple("Text"),
                parameters: vec![Parameter::Plain(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: "\"label\"",
                })],
                init_content: None,
            },
            Expression::StructInit {
                name: Name::simple("View"),
                parameters: vec![],
                init_content: Some(
                    vec![Expression::StructInit {
                        name: Name::simple("Text"),
                        parameters: vec![Parameter::Plain(Expression::Literal {
                            kind: LiteralKind::String,
                            lexeme: "\"nested\"",
                        })],
                        init_content: None,
                    }]
                    .into(),
                ),
            },
        ],
    )
}

fn test_struct_init_basic(
    statement: &str,
    expected_parameters: Vec<Parameter>,
    expected_body: Vec<Expression>,
) {
    let expected_body = if expected_body.is_empty() {
        None
    } else {
        Some(StructInitContent::from(expected_body))
    };
    let parse_tree = parse(tokenize(statement));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::StructInitExpression)),
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find StructInitExpression");
    let expression = Expression::from(node);
    match expression {
        Expression::StructInit {
            name,
            parameters,
            init_content: body,
        } => {
            assert_eq!(name, Name::simple("View"));
            assert_eq!(parameters, expected_parameters);
            assert_eq!(body, expected_body);
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
    let expression = find_first_expression(statement).expect("Cannot find Expression node");
    assert_eq!(
        expression,
        Expression::ChainingMethodInvocation {
            receiver: Box::new(Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "3",
            }),
            accessors: vec![
                Accessor {
                    identifier: "pow",
                    value: Some(Expression::Literal {
                        kind: LiteralKind::Integer,
                        lexeme: "2",
                    })
                },
                Accessor {
                    identifier: "abs",
                    value: None
                }
            ]
        }
    )
}

#[test]
fn test_const_use_qualified() {
    let program = "const text = book.content\n";
    let expression = find_first_expression(program).expect("Expression expected");
    let expected = Expression::ConstUse(Name::qualified(vec!["book", "content"]));
    assert_eq!(expression, expected)
}

#[test]
fn test_const_use_simple() {
    let program = "const text = book\n";
    let expression = find_first_expression(program).expect("Expression expected");
    let expected = Expression::ConstUse(Name::simple("book"));
    assert_eq!(expression, expected)
}

#[test]
fn test_field_access_from_block() {
    use crate::ast::Statement;
    let program = r#"
    const a = {
        3
    }.field
    "#;
    let expression = find_first_expression(program).expect("Expression expected");
    let expected = Expression::FieldAccess {
        receiver: Box::new(Expression::Block(
            vec![Statement::Expression(Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "3",
            })]
            .into(),
        )),
        field_names: vec!["field"],
    };
    assert_eq!(expression, expected)
}

#[test]
fn test_field_access_from_struct_init() {
    let program = r#"
    const a = Empty {
    }.field
    "#;
    let expression = find_first_expression(program).expect("Expression expected");
    let expected = Expression::FieldAccess {
        receiver: Box::new(Expression::StructInit {
            name: Name::simple("Empty"),
            parameters: vec![],
            init_content: None,
        }),
        field_names: vec!["field"],
    };
    assert_eq!(expression, expected)
}

#[test]
fn test_attribute_access_from_internal() {
    let program = r#"
        struct Id(number: Int) 
        struct person {
            const identifier = self.id.number
            const id = Id(3)
        }
    "#;
    let expression = find_first_expression(program).expect("Expression expected");
    let expected = Expression::FieldAccess {
        receiver: Box::new(Expression::SelfRef(None)),
        field_names: vec!["id", "number"],
    };
    assert_eq!(expression, expected)
}

#[test]
fn test_field_access_from_internal() {
    let program = r#"
        struct Id(number: Int)
        struct Person(id: Id) {
            const identifier = self.id.number
        }
        "#;
    let expression = find_first_expression(program).expect("Expression expected");
    let expected = Expression::FieldAccess {
        receiver: Box::new(Expression::SelfRef(None)),
        field_names: vec!["id", "number"],
    };
    assert_eq!(expression, expected)
}

#[test]
fn test_chaining_method_from_internal() {
    let program = r#"
    struct Id(number: Int)
    struct Person(id: Id) {
        const identifier = self.id.number(42)
    }
    "#;
    let expression = find_first_expression(program).expect("Expression expected");
    let expected = Expression::ChainingMethodInvocation {
        receiver: Box::new(Expression::FieldAccess {
            receiver: Box::new(Expression::SelfRef(None)),
            field_names: vec!["id"],
        }),
        accessors: vec![Accessor {
            identifier: "number",
            value: Some(Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "42",
            }),
        }],
    };
    assert_eq!(expression, expected)
}

#[test]
fn test_void_expression() {
    let actual = find_first_expression("const a = ()\n").expect("Expect Expression");
    let expected = Expression::Void;
    assert_eq!(actual, expected)
}

#[test]
fn test_collection_literal() {
    let actual = find_first_expression("const a = [1, 2, 3]\n").expect("Expect Expression");
    let expected = Expression::Collection(vec![
        Expression::Literal {
            kind: LiteralKind::Integer,
            lexeme: "1",
        },
        Expression::Literal {
            kind: LiteralKind::Integer,
            lexeme: "2",
        },
        Expression::Literal {
            kind: LiteralKind::Integer,
            lexeme: "3",
        },
    ]);
    assert_eq!(actual, expected)
}

#[test]
fn test_collection_literal_with_ending_comma() {
    let actual = find_first_expression("const a = [1,]\n").expect("Expect Expression");
    let expected = Expression::Collection(vec![Expression::Literal {
        kind: LiteralKind::Integer,
        lexeme: "1",
    }]);
    assert_eq!(actual, expected)
}

#[test]
fn test_empty_collection_literal() {
    let actual = find_first_expression("const a = []\n").expect("Expect Expression");
    let expected = Expression::Collection(vec![]);
    assert_eq!(actual, expected)
}

fn find_first_expression(program: &str) -> Option<Expression> {
    let parse_tree = parse(tokenize(program));
    BreadthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::Expression)),
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .map(Expression::from)
}
