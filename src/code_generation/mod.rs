use crate::ast::ConstantDeclaration;
use crate::env::scope::{DeclaredElement, GLOBAL_SCOPE};
use crate::env::Environment;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

mod value;
mod value_evaluator;
mod value_writer;

pub fn generate_code<P: AsRef<Path>>(environment: &Environment, output: P) {
    let entry = find_entry(environment);
    let mut evaluator = value_evaluator::ExpressionEvaluator::with_environment(environment);
    let evaluated_value = value_evaluator::evaluate(&mut evaluator, entry);

    // TODO: use std::io::Error instead
    let mut output = BufWriter::new(File::create(output).expect("Failed to create file"));
    value_writer::write(evaluator, evaluated_value, &mut output)
}

fn find_entry<'ast, 'a>(environment: &Environment<'ast, 'a>) -> &'ast ConstantDeclaration<'a> {
    let scope = environment.get_scope(GLOBAL_SCOPE);
    match scope.name_spaces.declared.get("Main") {
        Some(DeclaredElement::Constant(constant)) => constant,
        None => panic!("Main not declared in global scope"),
        Some(_) => panic!("Main should be declared as a constant"),
    }
}
