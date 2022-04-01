use super::super::expression_evaluator::ExpressionEvaluator;
use super::super::instance_evaluator::InstanceEvaluator;
use super::super::struct_evaluator::StructEvaluator;
use crate::ast::{
    abstract_tree, AbstractSyntaxTree, Expression, Parameter, StructDeclaration, StructInitContent,
};
use crate::code_generation::value::{Instance, Struct};
use crate::code_generation::value_evaluator::value::Value;
use crate::env::Environment;
use crate::formula_suppress::FormulaSuppress;
use crate::parser::parse;
use crate::tokenizer::tokenize;
use std::borrow::Cow;
use std::rc::Rc;

#[test]
fn test_struct_init_with_positional_param() {
    test_instance_fields(
        r#"
        struct S(field: String)
        
        const a = S("data")
        "#,
        vec![("field", Value::String(Cow::Borrowed("data")))],
    )
}

#[test]
fn test_struct_init_with_labelled_param() {
    test_instance_fields(
        r#"
        struct S(field: String)
        
        const a = S(field: "data")
        "#,
        vec![("field", Value::String(Cow::Borrowed("data")))],
    )
}

#[test]
fn test_struct_init_with_default_field() {
    test_instance_fields(
        r#"
        struct S(field: String = "data")
        
        const a = S()
        "#,
        vec![("field", Value::String(Cow::Borrowed("data")))],
    )
}

#[test]
fn test_positional_fields() {
    test_instance_fields(
        r#"
        struct S(a: Int, b: Int, c: Int)
        
        const a = S(1, 2, 3)
        "#,
        vec![
            ("a", Value::Int(1)),
            ("b", Value::Int(2)),
            ("c", Value::Int(3)),
        ],
    );
}

#[test]
fn test_ordered_fields() {
    test_instance_fields(
        r#"
        struct S(a: Int, b: Int, c: Int)
        
        const a = S(a: 1, b: 2, c: 3)
        "#,
        vec![
            ("a", Value::Int(1)),
            ("b", Value::Int(2)),
            ("c", Value::Int(3)),
        ],
    );
}

#[test]
fn test_disordered_fields() {
    test_instance_fields(
        r#"
        struct S(a: Int, b: Int, c: Int)

        const a = S(c: 3, a: 1, b: 2)
        "#,
        vec![
            ("a", Value::Int(1)),
            ("b", Value::Int(2)),
            ("c", Value::Int(3)),
        ],
    );
}

#[test]
fn test_overrides_default() {
    test_instance_fields(
        r#"
        struct S(a: Int, b: Int, c: Int=10)

        const a = S(c: 3, a: 1, b: 2)
        "#,
        vec![
            ("a", Value::Int(1)),
            ("b", Value::Int(2)),
            ("c", Value::Int(3)),
        ],
    );
}

#[test]
fn test_uses_default() {
    test_instance_fields(
        r#"
        struct S(a: Int, b: Int, c: Int=3)

        const a = S(b: 2, a: 1)
        "#,
        vec![
            ("a", Value::Int(1)),
            ("b", Value::Int(2)),
            ("c", Value::Int(3)),
        ],
    );
}

fn test_instance_fields(program: &str, expectations: Vec<(&str, Value)>) {
    let syntax_tree = abstract_tree(parse(tokenize(program)));
    let instance = get_struct_instance(&syntax_tree);
    for (field, value) in expectations {
        assert_eq!(instance.field(field), Some(value));
    }
}

#[test]
fn test_literal_attribute() {
    test_instance_attributes(
        r#"
    struct S() {
        const attr = 3 
    }
    const a = S()
    "#,
        vec![("attr", Value::Int(3))],
    )
}

#[test]
fn test_attribute_refers_to_field() {
    test_instance_attributes(
        r#"
    struct S(field: Int = 3) {
        const attr = self.field
    }
    const a = S()
    "#,
        vec![("attr", Value::Int(3))],
    )
}

#[test]
fn test_attribute_refers_to_attribute_without_self() {
    test_instance_attributes(
        r#"
    struct S(field: Int = 3) {
        const refer_field = self.field
        const attr = refer_field
    }
    const a = S()
    "#,
        vec![("attr", Value::Int(3))],
    )
}

#[test]
fn test_attribute_refers_to_attribute_with_self() {
    test_instance_attributes(
        r#"
    struct S(field: Int = 3) {
        const refer_field = self.field
        const attr = self.refer_field
    }
    const a = S()
    "#,
        vec![("attr", Value::Int(3))],
    )
}

fn test_instance_attributes(program: &str, expectations: Vec<(&str, Value)>) {
    let checkers = FormulaSuppress::all();
    checkers.suppress();

    let mut syntax_tree = [abstract_tree(parse(tokenize(program)))];
    let env = Environment::builder()
        .add_modules(&[vec![]])
        .generate_scopes(&mut syntax_tree)
        .resolve_names(&syntax_tree)
        .build();
    let instance = Rc::new(get_struct_instance(&syntax_tree[0]));
    let mut expr_resolver = ExpressionEvaluator::with_environment(&env);
    for (field, value) in expectations {
        assert_eq!(
            Rc::clone(&instance).attribute(&mut expr_resolver, field),
            Some(value)
        );
    }
}

fn get_struct_instance<'ast, 'a>(syntax_tree: &'ast AbstractSyntaxTree<'a>) -> Instance<'ast, 'a> {
    let env = Environment::default();
    let mut expr_resolver = ExpressionEvaluator::with_environment(&env);
    let struct_declaration = get_struct_declaration(syntax_tree).expect("Struct does not exist");
    let structure = std::rc::Rc::new(get_struct(&mut expr_resolver, struct_declaration));
    let (parameters, struct_init_content) =
        get_struct_init(syntax_tree).expect("StructInit not found");
    InstanceEvaluator::new(&mut expr_resolver, None).evaluate(
        std::rc::Rc::clone(&structure),
        &struct_declaration.fields,
        parameters,
        struct_init_content,
    )
}

fn get_struct_declaration<'ast, 'a>(
    syntax_tree: &'ast AbstractSyntaxTree<'a>,
) -> Option<&'ast StructDeclaration<'a>> {
    syntax_tree
        .compilation_unit
        .declarations
        .first()?
        .as_struct()
}

fn get_struct<'ast, 'a>(
    expr_resolver: &mut ExpressionEvaluator<'ast, 'a, '_>,
    declaration: &'ast StructDeclaration<'a>,
) -> Rc<Struct<'ast, 'a>> {
    StructEvaluator(expr_resolver).evaluate(declaration)
}

fn get_struct_init<'ast, 'a>(
    syntax_tree: &'ast AbstractSyntaxTree<'a>,
) -> Option<(&'ast [Parameter<'a>], &'ast Option<StructInitContent<'a>>)> {
    let constant = syntax_tree
        .compilation_unit
        .declarations
        .last()?
        .as_constant()?;
    match &constant.value {
        Expression::StructInit {
            name: _,
            parameters,
            init_content,
        } => Some((parameters, init_content)),
        _ => panic!("Not struct init"),
    }
}
