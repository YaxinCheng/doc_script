use super::super::abstract_tree;
use super::super::CompilationUnit;
use super::*;
use crate::ast::parameter::Parameter;
use crate::ast::Expression::{ChainingMethodInvocation, StructInit};
use crate::ast::{ConstantDeclaration, Declaration, Expression, Name};
use crate::tokenizer::LiteralKind::Integer;

#[test]
fn test_general_compilation() {
    let program = r#"
const main = View {
    Text("title")
    View {
        Text("body") ; Image(source: canada.lake)
    }
    .width(300)
}
"#;
    let parse_tree = parse(tokenize(program));
    let ast = abstract_tree(parse_tree);
    debug_assert_eq!(
        ast.compilation_unit,
        CompilationUnit {
            declarations: vec![Declaration::Constant(ConstantDeclaration {
                name: "main",
                value: Expression::StructInit {
                    name: Name::simple("View"),
                    parameters: vec![],
                    init_content: Some(
                        vec![
                            StructInit {
                                name: Name::simple("Text"),
                                parameters: vec![Parameter::Plain(Expression::Literal {
                                    kind: LiteralKind::String,
                                    lexeme: r#""title""#
                                })],
                                init_content: None
                            },
                            ChainingMethodInvocation {
                                receiver: Box::new(StructInit {
                                    name: Name::simple("View"),
                                    parameters: vec![],
                                    init_content: Some(
                                        vec![
                                            StructInit {
                                                name: Name::simple("Text"),
                                                parameters: vec![Parameter::Plain(
                                                    Expression::Literal {
                                                        kind: LiteralKind::String,
                                                        lexeme: r#""body""#
                                                    }
                                                )],
                                                init_content: None
                                            },
                                            StructInit {
                                                name: Name::simple("Image"),
                                                parameters: vec![Parameter::Labelled {
                                                    label: "source",
                                                    content: Expression::ConstUse(Name::qualified(
                                                        vec!["canada", "lake"]
                                                    ))
                                                }],
                                                init_content: None
                                            }
                                        ]
                                        .into()
                                    ),
                                }),
                                name: Name::simple("width"),
                                parameters: vec![Parameter::Plain(Expression::Literal {
                                    kind: Integer,
                                    lexeme: "300"
                                })]
                            }
                        ]
                        .into(),
                    )
                }
            })]
        }
    )
}
