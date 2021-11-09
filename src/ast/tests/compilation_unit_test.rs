use super::super::abstract_tree;
use super::super::CompilationUnit;
use super::*;
use crate::ast::Expression::{ChainingMethodInvocation, MethodInvocation, StructInit};
use crate::ast::{Declaration, Expression, Name, Parameter, Statement};
use crate::tokenizer::LiteralKind::Integer;

#[test]
fn test_general_compilation() {
    let program = r#"
const main = View {
    Text("title")
    View {
        Text("body") ; Image("lake")
    }
    .width(300)
}
"#;
    let parse_tree = parse(tokenize(program));
    let ast = abstract_tree(parse_tree);
    debug_assert_eq!(
        ast.compilation_unit,
        CompilationUnit {
            declarations: vec![Declaration::Constant(Statement::ConstantDeclaration {
                name: "main",
                value: Expression::StructInit {
                    name: Name::Simple("View"),
                    parameters: vec![],
                    body: vec![
                        MethodInvocation {
                            name: Name::Simple("Text"),
                            parameters: vec![Parameter::Plain(Expression::Literal {
                                kind: LiteralKind::String,
                                lexeme: r#""title""#
                            })],
                        },
                        ChainingMethodInvocation {
                            receiver: Box::new(StructInit {
                                name: Name::Simple("View"),
                                parameters: vec![],
                                body: vec![
                                    MethodInvocation {
                                        name: Name::Simple("Text"),
                                        parameters: vec![Parameter::Plain(Expression::Literal {
                                            kind: LiteralKind::String,
                                            lexeme: r#""body""#
                                        })],
                                    },
                                    MethodInvocation {
                                        name: Name::Simple("Image"),
                                        parameters: vec![Parameter::Plain(Expression::Literal {
                                            kind: LiteralKind::String,
                                            lexeme: r#""lake""#
                                        })]
                                    }
                                ]
                            }),
                            name: "width",
                            parameters: vec![Parameter::Plain(Expression::Literal {
                                kind: Integer,
                                lexeme: "300"
                            })]
                        }
                    ]
                }
            })]
        }
    )
}
