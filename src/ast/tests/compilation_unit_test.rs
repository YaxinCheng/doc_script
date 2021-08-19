use super::super::abstract_tree;
use super::super::CompilationUnit;
use super::*;
use crate::ast::Expression::StructInit;
use crate::ast::{Declaration, Expression, Name, Parameter};

#[test]
fn test_general_compilation() {
    let program = r#"
const main = View {
    Text("title"),
    View {
        Text("body") 
    }
};
"#
    .trim_start();
    let parse_tree = parse(tokenize(program));
    let ast = abstract_tree(parse_tree);
    debug_assert_eq!(
        ast.compilation_unit,
        CompilationUnit {
            declarations: vec![Declaration::Constant(Expression::Constant {
                name: "main",
                data: Box::new(Expression::StructInit {
                    name: Name::Simple("View"),
                    parameters: vec![],
                    body: vec![
                        StructInit {
                            name: Name::Simple("Text"),
                            parameters: vec![Parameter::Plain(Expression::Literal {
                                kind: LiteralKind::String,
                                lexeme: r#""title""#
                            })],
                            body: vec![]
                        },
                        StructInit {
                            name: Name::Simple("View"),
                            parameters: vec![],
                            body: vec![StructInit {
                                name: Name::Simple("Text"),
                                parameters: vec![Parameter::Plain(Expression::Literal {
                                    kind: LiteralKind::String,
                                    lexeme: r#""body""#
                                })],
                                body: vec![]
                            }]
                        }
                    ]
                })
            })]
        }
    )
}
