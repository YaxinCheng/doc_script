use crate::ast::abstract_tree;
use crate::env::Environment;
use crate::formula_suppress::FormulaSuppress;
use crate::parser::parse;
use crate::tokenizer::tokenize;

#[test]
#[should_panic]
fn test_cycle_reference() {
    test_constant_field_cycle_reference(
        r#"
        const a = b
        const b = a 
    "#,
        vec![vec![]],
    );
}

#[test]
#[should_panic]
fn test_cycle_reference_to_self() {
    test_constant_field_cycle_reference(
        r#"
        const a = a
    "#,
        vec![vec![]],
    );
}

#[test]
#[should_panic]
fn test_cycle_reference_to_self_with_full_name() {
    test_constant_field_cycle_reference(
        r#"
        const a = test.a
    "#,
        vec![vec!["test"]],
    );
}

#[test]
#[should_panic]
fn test_cycle_reference_with_full_name() {
    test_constant_field_cycle_reference(
        r#"
        const a = test.b
        const b = a
    "#,
        vec![vec!["test"]],
    );
}

#[test]
#[should_panic]
fn test_indirect_cycle_reference() {
    test_constant_field_cycle_reference(
        r#"
        const a = c
        const c = b
        const b = a
    "#,
        vec![vec![]],
    );
}

#[test]
#[should_panic]
fn test_field_reference_to_field() {
    test_constant_field_cycle_reference(
        r#"
    struct A(field1: String, field2: String = self.field1)
    "#,
        vec![vec![]],
    )
}

#[test]
#[should_panic]
fn test_field_reference_to_field_without_self() {
    test_constant_field_cycle_reference(
        r#"
    struct A(field1: String, field2: String = field1)
    "#,
        vec![vec![]],
    )
}

#[test]
fn test_field_use_constant_as_valid_default_value() {
    test_constant_field_cycle_reference(
        r#"
        const a = 5
        struct A(field: Int = a)
    "#,
        vec![vec![]],
    )
}

#[test]
#[should_panic]
fn test_constant_field_ref_cycle() {
    test_constant_field_cycle_reference(
        r#"
        const a = Test()
        const b = a.val
        struct Test(val: Int = b)
    "#,
        vec![vec![]],
    );
}

#[test]
#[should_panic]
fn test_constant_field_ref_without_using_default_value() {
    test_constant_field_cycle_reference(
        r#"
        const a = Test(3)
        const b = a.val
        struct Test(val: Int = b)
    "#,
        vec![vec![]],
    );
}

#[test]
fn test_constant_field_ref_valid() {
    test_constant_field_cycle_reference(
        r#"
        const a = Test()
        const b = 3
        struct Test(val: Int = b)
    "#,
        vec![vec![]],
    );
}

#[test]
#[should_panic]
fn test_cycle_ref_in_struct_init_param() {
    test_constant_field_cycle_reference(
        r#"
    struct Test(field: Int)
    
    const a = b
    const b = Test(a)
    "#,
        vec![vec![]],
    )
}

#[test]
#[should_panic]
fn test_cycle_ref_in_block() {
    test_constant_field_cycle_reference(
        r#"
    const a = {
        const b = a
        b
    }
    "#,
        vec![vec![]],
    )
}

#[test]
#[should_panic]
fn test_cycle_ref_in_struct_init_body() {
    test_constant_field_cycle_reference(
        r#"
    struct Test
    
    const a = b
    const b = Test {
        a
    }
    "#,
        vec![vec![]],
    )
}

fn test_constant_field_cycle_reference(program: &str, module_paths: Vec<Vec<&str>>) {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees)
        .build();
}
