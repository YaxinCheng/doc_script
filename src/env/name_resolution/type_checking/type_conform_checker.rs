use super::super::type_checking::TypeChecker;
use super::super::types::Types;
use crate::env::name_resolution::type_resolver;
use crate::env::TypedElement;

pub(in crate::env::name_resolution) struct TypeConformChecker<'ast, 'a, 'env, 'checker>(
    pub &'checker mut TypeChecker<'ast, 'a, 'env>,
);

impl<'ast, 'a, 'env, 'checker> TypeConformChecker<'ast, 'a, 'env, 'checker> {
    /// A conforms to B means A can be assigned to B
    /// String conforms to Any
    pub fn conforms(&mut self, source: &Types<'ast, 'a>, target: &Types<'ast, 'a>) -> bool {
        if source == target {
            true
        } else {
            match target {
                r#trait @ Types::Trait(_) => self.conforms_to_trait(source, r#trait),
                _ => false,
            }
        }
    }

    fn conforms_to_trait(
        &mut self,
        struct_type: &Types<'ast, 'a>,
        trait_type: &Types<'ast, 'a>,
    ) -> bool {
        for field in trait_type.fields() {
            if let Some(typed_element) = struct_type.access(field.name) {
                let expected_type =
                    type_resolver::resolve_type_name(self.0.environment, &field.field_type.0)
                        .expect("Expected trait field type not found");
                let found_type = match typed_element {
                    TypedElement::Field(found_field) => type_resolver::resolve_type_name(
                        self.0.environment,
                        &found_field.field_type.0,
                    )
                    .expect("Failed to find type for field"),
                    TypedElement::Constant(constant) => self.0.resolve_expression(&constant.value),
                };
                if found_type != expected_type {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod type_conform_checker_tests {
    use super::super::super::type_checking::TypeChecker;
    use super::TypeConformChecker;
    use crate::ast::{abstract_tree, AbstractSyntaxTree, Declaration};
    use crate::env::name_resolution::types::Types;
    use crate::env::Environment;
    use crate::parser::parse;
    use crate::tokenizer::tokenize;

    fn type_checker<'ast, 'a, 'env>(
        env: &'env Environment<'ast, 'a>,
    ) -> TypeChecker<'ast, 'a, 'env> {
        TypeChecker::with_environment(env)
    }

    #[test]
    fn test_type_equals() {
        let env = Environment::default();
        let mut type_checker = type_checker(&env);
        let mut conform_checker = TypeConformChecker(&mut type_checker);
        assert!(conform_checker.conforms(&Types::Int, &Types::Int))
    }

    #[test]
    fn test_struct_field_conform_trait() {
        test_struct_conform_trait("struct S(field1: Int, field2: String)\n")
    }

    #[test]
    fn test_struct_attributes_conform_trait() {
        test_struct_conform_trait(
            r#"
        struct S {
            const field1 = 3
            const field2 = ""
        }
        "#,
        )
    }

    #[test]
    fn test_struct_field_and_attribute_conform_trait() {
        test_struct_conform_trait(
            r#"
        struct S(field1: Int) {
            const field2 = ""
        }
        "#,
        )
    }

    #[test]
    #[should_panic]
    fn test_struct_not_conform_different_name() {
        test_struct_conform_trait(
            r#"
        struct S(field: Int) {
            const field2 = ""
        }
        "#,
        )
    }

    #[test]
    #[should_panic]
    fn test_struct_not_conform_different_type_field() {
        test_struct_conform_trait(
            r#"
        struct S(field1: String) {
            const field2 = ""
        }
        "#,
        )
    }

    #[test]
    #[should_panic]
    fn test_struct_not_conform_different_type_attribute() {
        test_struct_conform_trait(
            r#"
        struct S(field1: Int) {
            const field2 = 3
        }
        "#,
        )
    }

    fn test_struct_conform_trait(struct_declaration: &str) {
        let mut syntax_trees = [
            abstract_tree(parse(tokenize(struct_declaration))),
            abstract_tree(parse(tokenize("trait T(field1: Int, field2: String)\n"))),
        ];
        let module_paths = [vec![], vec![]];
        let env = Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .build();
        let mut type_checker = type_checker(&env);
        let mut conform_checker = TypeConformChecker(&mut type_checker);
        let struct_type = Types::Struct(first_declaration(&syntax_trees[0]).as_struct().unwrap());
        let trait_type = Types::Trait(first_declaration(&syntax_trees[1]).as_trait().unwrap());
        assert!(conform_checker.conforms(&struct_type, &trait_type))
    }

    #[test]
    fn test_trait_conform_trait() {
        let mut syntax_trees = [
            abstract_tree(parse(tokenize(
                "trait SubT(field: Int, another: String, field2: String)\n",
            ))),
            abstract_tree(parse(tokenize("trait T(field: Int, field2: String)\n"))),
        ];
        let module_paths = [vec![], vec![]];
        let env = Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .build();
        let mut type_checker = type_checker(&env);
        let mut conform_checker = TypeConformChecker(&mut type_checker);
        let struct_type = Types::Trait(first_declaration(&syntax_trees[0]).as_trait().unwrap());
        let trait_type = Types::Trait(first_declaration(&syntax_trees[1]).as_trait().unwrap());
        assert!(conform_checker.conforms(&struct_type, &trait_type))
    }

    #[test]
    fn test_any_trait() {
        let mut syntax_trees = [abstract_tree(parse(tokenize("trait Any\n")))];
        let module_paths = [vec![]];
        let env = Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .build();
        let mut type_checker = type_checker(&env);
        let mut conform_checker = TypeConformChecker(&mut type_checker);
        let trait_type = Types::Trait(first_declaration(&syntax_trees[0]).as_trait().unwrap());
        for source_type in [
            Types::Int,
            Types::Float,
            Types::String,
            Types::Void,
            Types::Bool,
            trait_type,
        ] {
            assert!(conform_checker.conforms(&source_type, &trait_type))
        }
    }

    fn first_declaration<'ast, 'a>(
        syntax_tree: &'ast AbstractSyntaxTree<'a>,
    ) -> &'ast Declaration<'a> {
        syntax_tree
            .compilation_unit
            .declarations
            .first()
            .expect("First declaration")
    }
}
