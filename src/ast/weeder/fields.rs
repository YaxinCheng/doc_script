use crate::ast::Field;
use std::collections::HashSet;

pub fn weed(fields: &[Field]) {
    let mut existing_labels = HashSet::new();
    for field in fields {
        if !existing_labels.insert(field.name) {
            panic!("Duplicate field ({}) appeared", field.name)
        }
    }
}

#[cfg(test)]
mod fields_weeder_tests {
    use super::weed;
    use crate::ast::{Field, Name, Type};

    fn field_type() -> Type<'static> {
        Type {
            name: Name::simple("type"),
            is_collection: false,
        }
    }

    #[test]
    fn test_no_duplicate() {
        let fields = [
            Field {
                name: "field1",
                field_type: field_type(),
                default_value: None,
            },
            Field {
                name: "field2",
                field_type: field_type(),
                default_value: None,
            },
        ];
        weed(&fields);
    }

    #[test]
    #[should_panic]
    fn test_duplicate() {
        let fields = [
            Field {
                name: "field1",
                field_type: field_type(),
                default_value: None,
            },
            Field {
                name: "field1",
                field_type: field_type(),
                default_value: None,
            },
        ];
        weed(&fields);
    }
}
