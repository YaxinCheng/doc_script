use crate::ast::{abstract_tree, AbstractSyntaxTree};
use crate::env::Environment;
use crate::formula_suppress::FormulaSuppress;
use crate::parser::parse;
use crate::stdlib;
use crate::tokenizer::tokenize;

fn get_syntax_trees<const N: usize>(source: [&'static str; N]) -> Vec<AbstractSyntaxTree<'static>> {
    stdlib::compiled_content()
        .into_iter()
        .chain(source.map(tokenize).map(parse).map(abstract_tree))
        .collect()
}

fn get_modules<const N: usize>(module_paths: [Vec<&'static str>; N]) -> Vec<Vec<&'static str>> {
    stdlib::module_paths()
        .into_iter()
        .chain(module_paths.into_iter())
        .collect()
}

#[test]
#[should_panic]
fn test_init_content_not_expected() {
    let mut syntax_trees = get_syntax_trees([r#"struct A
        const a = A {
            3
        }
        "#]);
    let module_paths = get_modules([vec![]]);
    Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}

#[test]
#[should_panic]
fn test_init_content_not_render() {
    let mut syntax_trees = get_syntax_trees([r#"struct A(children: [Render])
    const a = A {
        3
    }
    "#]);
    let module_paths = get_modules([vec![]]);
    Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}

#[test]
#[should_panic]
fn test_init_last_field_not_children() {
    let checkers = FormulaSuppress::allow_prelude_std();
    checkers.suppress();

    let mut syntax_trees = get_syntax_trees([r#"struct A(children: [Render], value: Int)
    const a = A {
        Page()
    }
    "#]);
    let module_paths = get_modules([vec![]]);
    Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}

#[test]
fn test_init_content() {
    let checkers = FormulaSuppress::allow_prelude_std();
    checkers.suppress();

    let mut syntax_trees = get_syntax_trees([r#"struct A(children: [Render])
    const a = A {
        Page()
    }
    "#]);
    let module_paths = get_modules([vec![]]);
    Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}

#[test]
fn test_init_with_parameters() {
    let checkers = FormulaSuppress::allow_prelude_std();
    checkers.suppress();

    let mut syntax_trees = get_syntax_trees([r#"struct A(value: Int, children: [Render])
    const a = A(3) {
        Page()
    }
    "#]);
    let module_paths = get_modules([vec![]]);
    Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}

#[test]
fn test_assign_children_to_children() {
    let checkers = FormulaSuppress::allow_prelude_std();
    checkers.suppress();

    let mut syntax_trees = get_syntax_trees([r#"struct A(children: [Render])
    struct B(children: [Render]) {
        const a = A(self.children)
    }
    const b = B {
        Page()
    }
    "#]);
    let module_paths = get_modules([vec![]]);
    Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}

#[test]
fn test_empty_children() {
    let checkers = FormulaSuppress::allow_prelude_std();
    checkers.suppress();

    let mut syntax_trees = get_syntax_trees([r#"struct A(children: [Render] = [])
    const a = A()
    "#]);
    let module_paths = get_modules([vec![]]);
    Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}
