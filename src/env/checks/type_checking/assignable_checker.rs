use super::type_resolver;
use crate::env::checks::type_checking::render_impl_checker::RenderImplChecker;
use crate::env::checks::type_checking::types::{Primitive, Types};
use crate::env::checks::type_checking::TypeChecker;
use crate::env::TypedElement;

/// This checker checks if source type can be assigned to target type
///
/// Mostly used in struct initialization when assigning values to a target field
/// (fields have expected types clarified as a requirement)
pub(in crate::env) struct AssignableChecker<'ast, 'a, 'env, 'checker>(
    pub &'checker mut TypeChecker<'ast, 'a, 'env>,
);

impl<'ast, 'a, 'env, 'checker> AssignableChecker<'ast, 'a, 'env, 'checker> {
    /// This function checks if source type is assignable to target type.
    /// In other words, if source type is assignable to target type,
    /// then fields with target type can hold values with source type
    pub fn check(&mut self, source: &Types<'ast, 'a>, target: &Types<'ast, 'a>) -> bool {
        source == target
            || Self::empty_assignability(source, target)
            || (matches!(target, Types::Trait(_)) && self.conforms_to_trait(source, target))
            || RenderImplChecker(self.0.environment).check(source, target)
    }

    fn empty_assignability(source: &Types<'ast, 'a>, target: &Types<'ast, 'a>) -> bool {
        matches!(source, Types::Primitive(Primitive::Void))
            && matches!(
                target,
                Types::PrimitiveCollection(_)
                    | Types::StructCollection(_)
                    | Types::TraitCollection(_)
            )
    }

    /// To make a type conforms to a trait,
    /// the source type need to implement all the required fields
    /// from the trait, with the same name and type
    ///
    /// # Note
    /// Both struct and trait can conform to another trait
    fn conforms_to_trait(
        &mut self,
        source_type: &Types<'ast, 'a>,
        trait_type: &Types<'ast, 'a>,
    ) -> bool {
        for field in trait_type.fields() {
            if let Some(typed_element) = source_type.access(field.name) {
                let expected_type = type_resolver::resolve_type_name(
                    self.0.environment,
                    &field.field_type.name,
                    field.field_type.is_collection,
                )
                .expect("Expected trait field type not found");
                let found_type = match typed_element {
                    TypedElement::Field(found_field) => type_resolver::resolve_type_name(
                        self.0.environment,
                        &found_field.field_type.name,
                        found_field.field_type.is_collection,
                    )
                    .expect("Failed to find type for field"),
                    TypedElement::Constant(constant) => self.0.resolve_expression(&constant.value),
                };
                if !self.check(&found_type, &expected_type) {
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
    use super::AssignableChecker;
    use crate::ast::{abstract_tree, AbstractSyntaxTree, Declaration};
    use crate::env::checks::type_checking::types::Types;
    use crate::env::checks::type_checking::TypeChecker;
    use crate::env::Environment;
    use crate::parser::parse;
    use crate::tests::FormulaSuppress;
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
        let mut conform_checker = AssignableChecker(&mut type_checker);
        assert!(conform_checker.check(&Types::INT, &Types::INT))
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
        let checkers = FormulaSuppress::all();
        checkers.suppress();

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
        let mut conform_checker = AssignableChecker(&mut type_checker);
        let struct_type = Types::Struct(first_declaration(&syntax_trees[0]).as_struct().unwrap());
        let trait_type = Types::Trait(first_declaration(&syntax_trees[1]).as_trait().unwrap());
        assert!(conform_checker.check(&struct_type, &trait_type))
    }

    #[test]
    fn test_trait_conform_trait() {
        let checkers = FormulaSuppress::all();
        checkers.suppress();

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
        let mut conform_checker = AssignableChecker(&mut type_checker);
        let struct_type = Types::Trait(first_declaration(&syntax_trees[0]).as_trait().unwrap());
        let trait_type = Types::Trait(first_declaration(&syntax_trees[1]).as_trait().unwrap());
        assert!(conform_checker.check(&struct_type, &trait_type))
    }

    #[test]
    fn test_any_trait() {
        let checkers = FormulaSuppress::all();
        checkers.suppress();

        let mut syntax_trees = [abstract_tree(parse(tokenize("trait Any\n")))];
        let module_paths = [vec![]];
        let env = Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .build();
        let mut type_checker = type_checker(&env);
        let mut conform_checker = AssignableChecker(&mut type_checker);
        let trait_type = Types::Trait(first_declaration(&syntax_trees[0]).as_trait().unwrap());
        for source_type in [
            Types::INT,
            Types::FLOAT,
            Types::STRING,
            Types::VOID,
            Types::BOOL,
            trait_type,
        ] {
            assert!(conform_checker.check(&source_type, &trait_type))
        }
    }

    #[test]
    #[should_panic]
    fn self_recursive_conform_traits() {
        let mut syntax_trees = [
            abstract_tree(parse(tokenize("trait Render(rendered: Render)\n"))),
            abstract_tree(parse(tokenize("struct View {\nconst rendered = self\n}\n"))),
        ];
        let module_paths = [vec![], vec![]];
        let env = Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .build();
        let mut type_checker = type_checker(&env);
        let mut conform_checker = AssignableChecker(&mut type_checker);
        let trait_type = Types::Trait(first_declaration(&syntax_trees[0]).as_trait().unwrap());
        let struct_type = Types::Struct(first_declaration(&syntax_trees[1]).as_struct().unwrap());
        assert!(!conform_checker.check(&struct_type, &trait_type))
    }

    #[test]
    fn test_two_level_recursive_conform() {
        let checkers = FormulaSuppress::all();
        checkers.suppress();

        let mut syntax_trees = [abstract_tree(parse(tokenize(
            r#"
                trait Id(number: Int)
                trait People(name: String, id: Id)
                struct IdImpl(number: Int = 42)
                // manager does not conform to People, even though IdImpl conforms to Id
                struct Manager(name: String, id: IdImpl) 
                struct Company(owner: People)
                const company = Company(owner: Manager("Name", IdImpl()))
                "#,
        )))];
        let module_paths = [vec![]];
        let _env = Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .validate(&syntax_trees)
            .build();
    }

    #[test]
    fn test_int_collection_assignability() {
        test_collection_assignability(
            r#"
        struct IntArray(elements: [Int])
        
        const arr = IntArray([1, 2, 3])
        "#,
        )
    }

    #[test]
    fn test_assign_empty_collection() {
        test_collection_assignability(
            r#"
        struct IntArray(elements: [Int])
        
        const arr = []
        "#,
        )
    }

    #[test]
    fn test_assign_void_to_collection() {
        test_collection_assignability(
            r#"
        struct IntArray(elements: [Int])
        
        const arr = ()
        "#,
        )
    }

    fn test_collection_assignability(program: &str) {
        let checkers = FormulaSuppress::all();
        checkers.suppress();

        let mut syntax_trees = [abstract_tree(parse(tokenize(program)))];
        let module_paths = [vec![]];
        Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .validate(&syntax_trees)
            .build();
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
