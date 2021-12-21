use super::super::{Expression, Statement};
use super::*;
use crate::ast::ConstantDeclaration;

#[test]
fn test_constant_declaration() {
    let parse_tree = parse(tokenize("const value = \"String\"\n"));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::ConstantDeclarationStatement)),
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .expect("Couldn't find ConstantDeclarationStatement node");
    let statement = Statement::from(node);
    assert!(matches!(
        statement,
        Statement::ConstantDeclaration(ConstantDeclaration { name: "value", .. })
    ));
    match statement {
        Statement::ConstantDeclaration(ConstantDeclaration { name, value }) => {
            assert_eq!(name, "value");
            assert!(matches!(
                value,
                Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""String""#
                }
            ));
        }
        expression => panic!("Unexpected expression: {:?}", expression),
    }
}
