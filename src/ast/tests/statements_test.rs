use super::super::{Expression, Statement};
use super::*;
use crate::ast::field::{Field, Type};
use crate::ast::{ConstantDeclaration, Name, StructDeclaration};

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
    let expression = Statement::from(node);
    assert!(matches!(
        expression,
        Statement::ConstantDeclaration(ConstantDeclaration { name: "value", .. })
    ));
    match expression {
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

#[test]
fn struct_declaration_test() {
    let program = r#"
struct Text {
    content: String = ""
    width: Int
}
"#;
    let parse_tree = parse(tokenize(program));
    let struct_init_node = DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::StructDeclarationStatement)),
        |node| node.children().unwrap_or_default(),
    )
    .map(Statement::from)
    .next()
    .expect("Unable to find StructDeclarationStatement");
    let expected = Statement::StructDeclaration(StructDeclaration {
        name: "Text",
        fields: vec![
            Field {
                name: "content",
                field_type: Type(Name::Simple("String")),
                default_value: Some(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""""#,
                }),
            },
            Field {
                name: "width",
                field_type: Type(Name::Simple("Int")),
                default_value: None,
            },
        ],
    });
    assert_eq!(struct_init_node, expected)
}
