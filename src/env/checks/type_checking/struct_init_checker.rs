use super::assignable_checker::AssignableChecker;
use crate::ast::{Field, Parameter};
use crate::env::checks::type_checking::types::Types;
use std::collections::{HashMap, VecDeque};

pub struct StructInitChecker<'ast, 'a, 'env, 'checker>(
    pub(in crate::env) AssignableChecker<'ast, 'a, 'env, 'checker>,
);

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Field `{0}` is not supplied")]
    FieldNotSupplied(String),
    #[error("Too many parameters provided.\nExpected: {expected}\nFound: {found}")]
    TooManyField { expected: usize, found: usize },
    #[error("Type mismatch for field `{}`.\nExpected: {expected}\nFound: {found}")]
    TypeMismatch {
        field: String,
        expected: String,
        found: String,
    },
}

impl<'ast, 'a, 'env, 'checker> StructInitChecker<'ast, 'a, 'env, 'checker> {
    pub fn check_parameters(
        &mut self,
        parameters: &[Parameter<'a>],
        parameter_types: Vec<Types<'ast, 'a>>,
        fields: &'ast [Field<'a>],
        field_types: Vec<Types<'ast, 'a>>,
    ) -> Result<(), Error> {
        if parameters.len() > fields.len() {
            Err(Error::TooManyField {
                expected: fields.len(),
                found: parameters.len(),
            })
        } else if parameters.is_empty() {
            if fields.is_empty() || fields[0].default_value.is_some() {
                Ok(())
            } else {
                let mandatory_field = &fields[0];
                Err(Error::FieldNotSupplied(mandatory_field.name.to_owned()))
            }
        } else if parameters[0].is_labelled() {
            self.check_labelled_parameters(parameters, &parameter_types, fields, field_types)
        } else {
            self.check_plain_parameters(parameter_types, fields, field_types)
        }
    }

    fn check_labelled_parameters(
        &mut self,
        parameters: &[Parameter],
        parameter_types: &[Types<'ast, 'a>],
        fields: &'ast [Field<'a>],
        field_types: Vec<Types<'ast, 'a>>,
    ) -> Result<(), Error> {
        let parameter_types = parameters
            .iter()
            .map(|parameter| match parameter {
                Parameter::Labelled { label, .. } => *label,
                Parameter::Plain(_) => unreachable!("Cannot mix labelled and plain parameter"),
            })
            .zip(parameter_types)
            .collect::<HashMap<_, _>>();
        for (field, field_type) in fields.iter().zip(&field_types) {
            if let Some(parameter_type) = parameter_types.get(field.name) {
                if !self.0.check(parameter_type, field_type) {
                    return Err(Error::TypeMismatch {
                        field: field.name.to_owned(),
                        expected: format!("{:?}", field_type),
                        found: format!("{:?}", parameter_type),
                    });
                }
            } else if field.default_value.is_none() {
                return Err(Error::FieldNotSupplied(field.name.to_owned()));
            }
        }
        Ok(())
    }

    fn check_plain_parameters(
        &mut self,
        parameter_types: Vec<Types<'ast, 'a>>,
        fields: &'ast [Field<'a>],
        field_types: Vec<Types<'ast, 'a>>,
    ) -> Result<(), Error> {
        let mut parameter_types = VecDeque::from(parameter_types);
        for (field, expected_type) in fields.iter().zip(&field_types) {
            if parameter_types.is_empty() {
                return match field.default_value {
                    Some(_) => Ok(()),
                    None => Err(Error::FieldNotSupplied(field.name.to_owned())),
                };
            } else if self.0.check(&parameter_types[0], expected_type) {
                parameter_types.pop_front();
            } else if field.default_value.is_none() {
                return Err(Error::TypeMismatch {
                    field: field.name.to_owned(),
                    expected: format!("{:?}", expected_type),
                    found: format!("{:?}", &parameter_types[0]),
                });
            }
        }
        if parameter_types.is_empty() {
            Ok(())
        } else {
            unreachable!("Unable to match parameters with required fields")
        }
    }
}

#[cfg(test)]
mod struct_init_checker_tests {
    use super::{Error, StructInitChecker};
    use crate::ast::{Expression, Field, Name, Parameter, Type};
    use crate::env::checks::type_checking::assignable_checker::AssignableChecker;
    use crate::env::checks::type_checking::types::Types;
    use crate::env::checks::type_checking::TypeChecker;
    use crate::env::Environment;

    fn field(name: &str, default_value: bool) -> Field {
        let default_value = match default_value {
            false => None,
            true => Some(Expression::ConstUse(Name::simple("test"))),
        };
        Field {
            name,
            field_type: Type(Name::simple("not important")),
            default_value,
        }
    }

    #[test]
    fn test_type_plain_parameters() {
        let check_outcome = check_plain_parameters(
            vec![field("field1", false), field("field2", false)],
            vec![Types::Int, Types::String],
            vec![Types::Int, Types::String],
        );
        assert!(check_outcome.is_ok())
    }

    #[test]
    fn test_type_plain_mismatches() {
        let check_outcome = check_plain_parameters(
            vec![field("field1", false), field("field2", false)],
            vec![Types::Int, Types::String],
            vec![Types::String, Types::Int],
        );
        assert_eq!(
            check_outcome,
            Err(Error::TypeMismatch {
                field: "field1".into(),
                expected: "Int".into(),
                found: "String".into()
            })
        )
    }

    #[test]
    fn test_type_plain_with_default_parameter() {
        let check_outcome = check_plain_parameters(
            vec![field("field1", false), field("field2", true)],
            vec![Types::Int, Types::String],
            vec![Types::Int],
        );
        assert!(check_outcome.is_ok())
    }

    #[test]
    fn test_type_plain_modifying_default_parameter() {
        let check_outcome = check_plain_parameters(
            vec![field("field1", true), field("field2", true)],
            vec![Types::Int, Types::String],
            vec![Types::Int],
        );
        assert!(check_outcome.is_ok())
    }

    #[test]
    fn test_type_plain_modifying_second_default_parameter() {
        let check_outcome = check_plain_parameters(
            vec![field("field1", true), field("field2", true)],
            vec![Types::Int, Types::String],
            vec![Types::String],
        );
        assert!(check_outcome.is_ok())
    }

    #[test]
    fn test_type_plain_default_field_matches() {
        let check_outcome = check_plain_parameters(
            vec![
                field("field1", false),
                field("field2", true),
                field("field3", true),
            ],
            vec![Types::Int, Types::String, Types::Int],
            vec![Types::Int, Types::Int],
        );
        assert!(check_outcome.is_ok())
    }

    #[test]
    fn test_type_plain_default_field_not_supplied() {
        let check_outcome = check_plain_parameters(
            vec![field("field1", false), field("field2", true)],
            vec![Types::Int, Types::String],
            vec![Types::String],
        );
        assert_eq!(
            check_outcome,
            Err(Error::TypeMismatch {
                field: "field1".into(),
                expected: "Int".into(),
                found: "String".into()
            })
        )
    }

    fn check_plain_parameters(
        fields: Vec<Field>,
        field_types: Vec<Types>,
        parameter_types: Vec<Types>,
    ) -> Result<(), Error> {
        let env = Environment::default();
        let mut type_checker = TypeChecker::with_environment(&env);
        let type_conform_checker = AssignableChecker(&mut type_checker);
        let mut struct_init_checker = StructInitChecker(type_conform_checker);
        struct_init_checker.check_plain_parameters(parameter_types, &fields, field_types)
    }

    #[test]
    fn test_labelled_parameters_in_order() {
        let check_res = check_labelled_parameters(
            vec![field("field1", false), field("field2", false)],
            vec![Types::Int, Types::String],
            vec![parameter("field1"), parameter("field2")],
            vec![Types::Int, Types::String],
        );
        assert!(check_res.is_ok())
    }

    #[test]
    fn test_labelled_parameters_in_reverse_order() {
        let check_res = check_labelled_parameters(
            vec![field("field1", false), field("field2", false)],
            vec![Types::Int, Types::String],
            vec![parameter("field2"), parameter("field1")],
            vec![Types::String, Types::Int],
        );
        assert!(check_res.is_ok())
    }

    #[test]
    fn test_labelled_parameters_with_defaults() {
        let check_res = check_labelled_parameters(
            vec![
                field("field1", false),
                field("field2", true),
                field("field3", true),
            ],
            vec![Types::Int, Types::String, Types::Int],
            vec![parameter("field2"), parameter("field1")],
            vec![Types::String, Types::Int],
        );
        assert!(check_res.is_ok())
    }

    #[test]
    fn test_labelled_parameters_missing_mandatory() {
        let check_res = check_labelled_parameters(
            vec![
                field("field1", false),
                field("field2", true),
                field("field3", true),
            ],
            vec![Types::Int, Types::String, Types::Int],
            vec![parameter("field2"), parameter("field3")],
            vec![Types::String, Types::Int],
        );
        assert_eq!(check_res, Err(Error::FieldNotSupplied("field1".into())))
    }

    #[test]
    fn test_labelled_parameters_type_mismatches() {
        let check_res = check_labelled_parameters(
            vec![field("field1", false), field("field2", false)],
            vec![Types::Int, Types::String],
            vec![parameter("field1"), parameter("field2")],
            vec![Types::String, Types::String],
        );
        assert_eq!(
            check_res,
            Err(Error::TypeMismatch {
                field: "field1".into(),
                expected: "Int".into(),
                found: "String".into()
            })
        )
    }

    #[test]
    fn test_too_many_parameters() {
        let fields = vec![field("field1", false)];
        let parameters = vec![parameter("field1"), parameter("field2")];
        let env = Environment::default();
        let mut type_checker = TypeChecker::with_environment(&env);
        let type_conform_checker = AssignableChecker(&mut type_checker);
        let check_res = StructInitChecker(type_conform_checker).check_parameters(
            &parameters,
            vec![Types::Int, Types::String],
            &fields,
            vec![Types::Int],
        );
        assert_eq!(
            check_res,
            Err(Error::TooManyField {
                expected: 1,
                found: 2
            })
        )
    }

    #[test]
    fn test_empty_input() {
        let fields = vec![field("field1", false), field("field2", false)];
        let parameters = vec![];
        let env = Environment::default();
        let mut type_checker = TypeChecker::with_environment(&env);
        let type_conform_checker = AssignableChecker(&mut type_checker);
        let check_res = StructInitChecker(type_conform_checker).check_parameters(
            &parameters,
            vec![],
            &fields,
            vec![Types::Int, Types::String],
        );
        assert_eq!(check_res, Err(Error::FieldNotSupplied("field1".into())))
    }

    fn parameter(label: &str) -> Parameter {
        Parameter::Labelled {
            label,
            content: Expression::ConstUse(Name::simple("test")),
        }
    }

    fn check_labelled_parameters(
        fields: Vec<Field>,
        field_types: Vec<Types>,
        parameters: Vec<Parameter>,
        parameter_types: Vec<Types>,
    ) -> Result<(), Error> {
        let env = Environment::default();
        let mut type_checker = TypeChecker::with_environment(&env);
        let type_conform_checker = AssignableChecker(&mut type_checker);
        StructInitChecker(type_conform_checker).check_labelled_parameters(
            &parameters,
            &parameter_types,
            &fields,
            field_types,
        )
    }
}
