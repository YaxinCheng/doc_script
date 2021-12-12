use super::super::{Expression, Statement};
use super::*;
use crate::ast::field::{Field, Type};
use crate::ast::{ConstantDeclaration, Name, StructBody, StructDeclaration};

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

#[test]
fn struct_declaration_test() {
    let program = r#"
struct Square(
    content: String = "",
    width: Int
) {
    const height = width
}
"#;
    let struct_declaration = get_struct(program);
    let expected = Statement::StructDeclaration(StructDeclaration {
        name: "Square",
        fields: vec![
            Field {
                name: "content",
                field_type: Type(Name::simple("String")),
                default_value: Some(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""""#,
                }),
            },
            Field {
                name: "width",
                field_type: Type(Name::simple("Int")),
                default_value: None,
            },
        ],
        body: vec![ConstantDeclaration {
            name: "height",
            value: Expression::ConstUse(Name::simple("width")),
        }]
        .into(),
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
fn struct_declaration_without_body_test() {
    let program = r#"
struct Square(
    content: String = "",
    width: Int
)
"#;
    let struct_declaration = get_struct(program);
    let expected = Statement::StructDeclaration(StructDeclaration {
        name: "Square",
        fields: vec![
            Field {
                name: "content",
                field_type: Type(Name::simple("String")),
                default_value: Some(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""""#,
                }),
            },
            Field {
                name: "width",
                field_type: Type(Name::simple("Int")),
                default_value: None,
            },
        ],
        body: StructBody::default(),
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
fn struct_declaration_without_fields() {
    let program = r#"
struct Square {
    const side = 3
}
"#;
    let struct_declaration = get_struct(program);
    let expected = Statement::StructDeclaration(StructDeclaration {
        name: "Square",
        fields: vec![],
        body: vec![ConstantDeclaration {
            name: "side",
            value: Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "3",
            },
        }]
        .into(),
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
fn struct_declaration_without_fields_or_body() {
    let program = r#"
struct Square
"#;
    let struct_declaration = get_struct(program);
    let expected = Statement::StructDeclaration(StructDeclaration {
        name: "Square",
        fields: vec![],
        body: StructBody::default(),
    });
    assert_eq!(struct_declaration, expected)
}

fn get_struct(program: &str) -> Statement {
    let parse_tree = parse(tokenize(program));
    DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::StructDeclarationStatement)),
        |node| node.children().unwrap_or_default(),
    )
    .map(Statement::from)
    .next()
    .expect("Unable to find StructDeclarationStatement")
}
