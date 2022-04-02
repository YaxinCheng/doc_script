use crate::ast::{abstract_tree, ConstantDeclaration, Name, StructDeclaration};
use crate::env::scope::{Scoped, GLOBAL_SCOPE};
use crate::env::Environment;
use crate::parser::parse;
use crate::tests::FormulaSuppress;
use crate::tokenizer::tokenize;

macro_rules! try_block {
    ($kind: ty, $block: expr) => {{
        let __try_block = || -> Option<$kind> { $block };
        __try_block().unwrap()
    }};
}

#[test]
fn resolve_module_constant() {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize("const a = test.target\n"))),
        abstract_tree(parse(tokenize("const target = 3\n"))),
    ];
    let module_paths = vec![vec![], vec!["test"]];
    let mut name = Name::qualified(["test", "target"]);
    name.set_scope(GLOBAL_SCOPE);
    let resolved = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build()
        .resolved_names;
    let actual = try_block!(
        &ConstantDeclaration,
        resolved.get(&name)?.as_constant().copied()
    );
    let expected = try_block!(
        &ConstantDeclaration,
        syntax_trees
            .last()?
            .compilation_unit
            .declarations
            .last()?
            .as_constant()
    );
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn resolve_module_struct() {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize("const a = empty.Empty()\n"))),
        abstract_tree(parse(tokenize("struct Empty\n"))),
    ];
    let module_paths = vec![vec![], vec!["empty"]];
    let mut target_name = Name::qualified(["empty", "Empty"]);
    target_name.set_scope(GLOBAL_SCOPE);
    let resolved_names = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build()
        .resolved_names;
    let actual = try_block!(
        &StructDeclaration,
        resolved_names.get(&target_name)?.as_struct().copied()
    );
    let expected = try_block!(
        &StructDeclaration,
        syntax_trees
            .last()?
            .compilation_unit
            .declarations
            .last()?
            .as_struct()
    );
    assert!(std::ptr::eq(actual, expected));
}
