use crate::ast::{
    ConstantDeclaration, Declaration, Expression, Field, Name, StructDeclaration, TraitDeclaration,
    Type,
};
use crate::parser::{parse, NodeKind};
use crate::search::DepthFirst;
use crate::tokenizer::{tokenize, LiteralKind};

#[test]
fn struct_declaration_test() {
    let program = r#"
struct Square(
    width: Int,
    content: String = ""
) {
    const height = width
}
"#;
    let struct_declaration = get_struct(program);
    let expected = Declaration::Struct(StructDeclaration {
        name: "Square",
        fields: vec![
            Field {
                name: "width",
                field_type: Type {
                    name: Name::simple("Int"),
                    is_collection: false,
                },
                default_value: None,
            },
            Field {
                name: "content",
                field_type: Type {
                    name: Name::simple("String"),
                    is_collection: false,
                },
                default_value: Some(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""""#,
                }),
            },
        ],
        body: Some(
            vec![ConstantDeclaration {
                name: "height",
                value: Expression::ConstUse(Name::simple("width")),
            }]
            .into(),
        ),
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
fn struct_declaration_without_body_test() {
    let program = r#"
struct Square(
    width: Int,
    content: String = "",
)
"#;
    let struct_declaration = get_struct(program);
    let expected = Declaration::Struct(StructDeclaration {
        name: "Square",
        fields: vec![
            Field {
                name: "width",
                field_type: Type {
                    name: Name::simple("Int"),
                    is_collection: false,
                },
                default_value: None,
            },
            Field {
                name: "content",
                field_type: Type {
                    name: Name::simple("String"),
                    is_collection: false,
                },
                default_value: Some(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""""#,
                }),
            },
        ],
        body: None,
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
fn struct_declaration_multiple_fields() {
    let program = r#"
struct Square(
    width: Int,
    height: Int,
    content: String = "",
    id: Int = 0
)
"#;
    let struct_declaration = get_struct(program);
    let expected = Declaration::Struct(StructDeclaration {
        name: "Square",
        fields: vec![
            Field {
                name: "width",
                field_type: Type {
                    name: Name::simple("Int"),
                    is_collection: false,
                },
                default_value: None,
            },
            Field {
                name: "height",
                field_type: Type {
                    name: Name::simple("Int"),
                    is_collection: false,
                },
                default_value: None,
            },
            Field {
                name: "content",
                field_type: Type {
                    name: Name::simple("String"),
                    is_collection: false,
                },
                default_value: Some(Expression::Literal {
                    kind: LiteralKind::String,
                    lexeme: r#""""#,
                }),
            },
            Field {
                name: "id",
                field_type: Type {
                    name: Name::simple("Int"),
                    is_collection: false,
                },
                default_value: Some(Expression::Literal {
                    kind: LiteralKind::Integer,
                    lexeme: r#"0"#,
                }),
            },
        ],
        body: None,
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
#[should_panic]
fn struct_default_field_comes_first() {
    let program = r#"
struct Square(
    content: String = "",
    width: Int,
)
    "#;
    get_struct(program);
}

#[test]
fn struct_declaration_without_fields() {
    let program = r#"
struct Square {
    const side = 3
}
"#;
    let struct_declaration = get_struct(program);
    let expected = Declaration::Struct(StructDeclaration {
        name: "Square",
        fields: vec![],
        body: Some(
            vec![ConstantDeclaration {
                name: "side",
                value: Expression::Literal {
                    kind: LiteralKind::Integer,
                    lexeme: "3",
                },
            }]
            .into(),
        ),
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
fn struct_declaration_without_fields_or_body() {
    let program = r#"
struct Square
"#;
    let struct_declaration = get_struct(program);
    let expected = Declaration::Struct(StructDeclaration {
        name: "Square",
        fields: vec![],
        body: None,
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
fn test_struct_with_collection_field() {
    let program = r#"
    struct IntArray(elements: [Int])
    "#;
    let struct_declaration = get_struct(program);
    let expected = Declaration::Struct(StructDeclaration {
        name: "IntArray",
        fields: vec![Field {
            name: "elements",
            field_type: Type {
                name: Name::simple("Int"),
                is_collection: true,
            },
            default_value: None,
        }],
        body: None,
    });
    assert_eq!(struct_declaration, expected)
}

#[test]
fn test_struct_with_collection_field_with_default_value() {
    let program = r#"
    struct IntArray(elements: [Int] = [1,])
    "#;
    let struct_declaration = get_struct(program);
    let expected = Declaration::Struct(StructDeclaration {
        name: "IntArray",
        fields: vec![Field {
            name: "elements",
            field_type: Type {
                name: Name::simple("Int"),
                is_collection: true,
            },
            default_value: Some(Expression::Collection(vec![Expression::Literal {
                kind: LiteralKind::Integer,
                lexeme: "1",
            }])),
        }],
        body: None,
    });
    assert_eq!(struct_declaration, expected)
}

fn get_struct(program: &str) -> Declaration {
    let parse_tree = parse(tokenize(program));
    DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::StructDeclarationStatement)),
        |node| node.children().unwrap_or_default(),
    )
    .map(Declaration::from)
    .next()
    .expect("Unable to find StructDeclarationStatement")
}

#[test]
fn test_trait_declaration_empty_no_bracket() {
    let program = "trait Trait\n";
    let actual = get_trait(program);
    let expected = Declaration::Trait(TraitDeclaration {
        name: "Trait",
        required: vec![],
    });
    assert_eq!(actual, expected)
}

#[test]
fn test_trait_declaration_empty() {
    let program = "trait Trait()\n";
    let actual = get_trait(program);
    let expected = Declaration::Trait(TraitDeclaration {
        name: "Trait",
        required: vec![],
    });
    assert_eq!(actual, expected)
}

#[test]
fn test_trait_declaration_fields() {
    let program = "trait Trait(first: Int, second: String)\n";
    let actual = get_trait(program);
    let expected = Declaration::Trait(TraitDeclaration {
        name: "Trait",
        required: vec![
            Field {
                name: "first",
                field_type: Type {
                    name: Name::simple("Int"),
                    is_collection: false,
                },
                default_value: None,
            },
            Field {
                name: "second",
                field_type: Type {
                    name: Name::simple("String"),
                    is_collection: false,
                },
                default_value: None,
            },
        ],
    });
    assert_eq!(actual, expected)
}

fn get_trait(program: &str) -> Declaration {
    let parse_tree = parse(tokenize(program));
    DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::TraitDeclarationStatement)),
        |node| node.children().unwrap_or_default(),
    )
    .map(Declaration::from)
    .next()
    .expect("Unable to find StructDeclarationStatement")
}
