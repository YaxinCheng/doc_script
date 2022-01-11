use crate::ast::ConstantDeclaration;
use crate::env::scope::{DeclaredElement, GLOBAL_SCOPE};
use crate::env::Environment;

mod value_evaluator;

pub fn generate_code(environment: &Environment) -> String {
    let entry = find_entry(environment);
    let _evaluated_value = value_evaluator::evaluate(environment, entry);
    todo!("Serialize the resolved value to string")
}

fn find_entry<'ast, 'a>(environment: &Environment<'ast, 'a>) -> &'ast ConstantDeclaration<'a> {
    let scope = environment.get_scope(GLOBAL_SCOPE);
    match scope.name_spaces.declared.get("Main") {
        Some(DeclaredElement::Constant(constant)) => constant,
        None => panic!("Main not declared in global scope"),
        Some(_) => panic!("Main should be declared as a constant"),
    }
}
