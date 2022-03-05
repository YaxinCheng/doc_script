use super::try_block;
use crate::ast::{
    abstract_tree, ConstantDeclaration, Expression, StructDeclaration, TraitDeclaration,
};
use crate::env::checks::type_checking::types::Types;
use crate::env::checks::type_checking::TypeChecker;
use crate::env::Environment;
use crate::parser::parse;
use crate::tests::FormulaSuppress;
use crate::tokenizer::{tokenize, LiteralKind};

#[test]
fn test_int() {
    test_literals(LiteralKind::Integer, Types::Int)
}

#[test]
fn test_binary() {
    test_literals(LiteralKind::Binary, Types::Int)
}

#[test]
fn test_float() {
    test_literals(LiteralKind::Floating, Types::Float)
}

#[test]
fn test_string() {
    test_literals(LiteralKind::String, Types::String)
}

#[test]
fn test_bool() {
    test_literals(LiteralKind::Boolean, Types::Bool)
}

fn test_literals(kind: LiteralKind, expected: Types) {
    let expression = Expression::Literal { kind, lexeme: "" };
    let env = Environment::default();
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(&expression);
    assert_eq!(actual, expected)
}

#[test]
fn test_resolve_struct() {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
        struct Empty
        const a = Empty()
        "#,
    )))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_expression = try_block!(
        &Expression,
        Some(
            &syntax_trees
                .first()?
                .compilation_unit
                .declarations
                .last()?
                .as_constant()?
                .value
        )
    );
    let target_struct = try_block!(
        &StructDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()
    );
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(target_expression);
    let expected = Types::Struct(target_struct);
    assert_eq!(actual, expected)
}

#[test]
fn test_resolve_from_block() {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
        const a = {
            const b = 3
            b
        }
        "#,
    )))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_block = try_block!(
        &Expression,
        Some(
            &syntax_trees
                .first()?
                .compilation_unit
                .declarations
                .first()?
                .as_constant()?
                .value
        )
    );
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(target_block);
    let expected = Types::Int;
    assert_eq!(actual, expected)
}

#[test]
fn test_resolve_field_access_from_block() {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
        struct A(field: String)
        const a = {
            A("test")
        }.field
        "#,
    )))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_block = try_block!(
        &Expression,
        Some(
            &syntax_trees
                .first()?
                .compilation_unit
                .declarations
                .last()?
                .as_constant()?
                .value
        )
    );
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(target_block);
    let expected = Types::String;
    assert_eq!(actual, expected)
}

#[test]
fn test_type_field_access_internal() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const a = self.field
        }
        "#,
    );
}

#[test]
fn test_type_attribute_access_internal() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const b = self.field
            const a = b
        }
        "#,
    );
}

#[test]
#[should_panic]
fn test_type_field_access_internal_without_self() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const a = field
        }
        "#,
    );
}

#[test]
fn test_type_attribute_access_internal_with_self() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const b = self.field
            const a = self.b
        }
        "#,
    );
}

#[test]
#[should_panic]
fn test_type_attribute_access_internal_directly_from_type() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const b = self.field
            const a = A.b
        }
        "#,
    );
}

#[test]
#[should_panic]
fn test_type_field_access_internal_directly_from_type() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const a = A.field
        }
        "#,
    );
}

#[test]
#[should_panic]
fn test_type_constant_points_to_struct() {
    test_type_access_internal(
        r#"
        struct A(field: String) 
        const a = A
        "#,
    );
}

fn test_type_access_internal(program: &str) {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_constant = try_block!(
        &ConstantDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()?
            .body
            .as_ref()?
            .attributes
            .first()
    );

    let actual =
        TypeChecker::with_environment(&env).test_resolve_expression(&target_constant.value);
    let expected = Types::String;
    assert_eq!(actual, expected)
}

#[test]
fn test_access_field_with_trait_type() {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let program = r#"
    trait Trait(value: Int)
    struct Impl(value: Int)
    struct TestType(field: Trait)
    const test = TestType(Impl(42)).field
    "#;
    let module_paths = vec![vec![]];
    let mut syntax_trees = [abstract_tree(parse(tokenize(program)))];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let trait_declaration = try_block!(
        &TraitDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_trait()
    );
    let target_constant = try_block!(
        &ConstantDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .last()?
            .as_constant()
    );
    let actual =
        TypeChecker::with_environment(&env).test_resolve_expression(&target_constant.value);
    let expected = Types::Trait(trait_declaration);
    assert_eq!(actual, expected)
}

#[test]
#[should_panic]
fn access_undeclared_field_from_trait() {
    let program = r#"
    trait Trait(value: Int)
    struct Impl(value: Int, hidden: String)
    struct TestType(field: Trait)
    const test = TestType(Impl(42)).hidden
    "#;
    let module_paths = vec![vec![]];
    let mut syntax_trees = [abstract_tree(parse(tokenize(program)))];
    let _env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees)
        .build();
}

#[test]
fn test_void() {
    let void_expr = Expression::Void;
    let env = Environment::default();
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(&void_expr);
    assert_eq!(actual, Types::Void)
}

#[test]
fn test_void_type_full() {
    test_void_type_in_const(
        r#"
    const marker = ()
    "#,
    );
}

fn test_void_type_in_const(program: &str) {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let module_paths = vec![vec![]];
    let mut syntax_trees = [abstract_tree(parse(tokenize(program)))];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_constant = try_block!(
        &ConstantDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_constant()
    );
    let actual =
        TypeChecker::with_environment(&env).test_resolve_expression(&target_constant.value);
    let expected = Types::Void;
    assert_eq!(actual, expected)
}

#[test]
fn test_void_type_impl_trait() {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = [
        abstract_tree(parse(tokenize("trait Test(marker: Void)\n"))),
        abstract_tree(parse(tokenize("struct Collection(test: Test)\n"))),
        abstract_tree(parse(tokenize(
            r#"struct Impl(data: String) { 
            const marker = () 
            }
            "#,
        ))),
        abstract_tree(parse(tokenize(
            r#"
            const collection = Collection(Impl("test"))
            "#,
        ))),
    ];
    let module_paths = [vec![], vec![], vec![], vec![]];
    let _env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees)
        .build();
}
