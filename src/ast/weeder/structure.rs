use crate::ast::StructDeclaration;
use std::collections::HashSet;

pub fn weed(structure: &StructDeclaration) {
    if structure.body.is_none() {
        return;
    }
    let field_names = structure
        .fields
        .iter()
        .map(|field| field.name)
        .collect::<HashSet<_>>();
    let conflict_name = structure
        .body
        .as_ref()
        .unwrap()
        .attributes
        .iter()
        .find(|attribute| field_names.contains(attribute.name))
        .map(|attribute| attribute.name);
    if let Some(conflict_name) = conflict_name {
        panic!(
            "struct {} has both attribute and field with name {}",
            structure.name, conflict_name
        )
    }
}

#[cfg(test)]
mod struct_weeder_tests {
    use super::weed;
    use crate::ast::{ConstantDeclaration, Expression, Field, Name, StructDeclaration, Type};

    fn field(name: &str) -> Field {
        Field {
            name,
            field_type: Type(Name::simple("type")),
            default_value: None,
        }
    }

    #[test]
    #[should_panic]
    fn test_conflicting_name() {
        let structure = StructDeclaration {
            name: "test",
            fields: vec![field("test"), field("field")],
            body: Some(
                [ConstantDeclaration {
                    name: "test",
                    value: Expression::ConstUse(Name::simple("test")),
                }]
                .into_iter()
                .collect(),
            ),
        };
        weed(&structure)
    }

    #[test]
    fn test_no_conflicting_name() {
        let structure = StructDeclaration {
            name: "test",
            fields: vec![field("test"), field("field")],
            body: Some(
                [ConstantDeclaration {
                    name: "test1",
                    value: Expression::ConstUse(Name::simple("test")),
                }]
                .into_iter()
                .collect(),
            ),
        };
        weed(&structure)
    }
}
