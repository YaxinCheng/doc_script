use crate::env::checks::type_checking::types::Types;
use crate::env::scope::DeclaredElement;
use crate::env::Environment;

const STD_ESSENTIAL: &[&str] = &["std", "essential"];

pub fn render<'ast, 'a>(environment: &Environment<'ast, 'a>) -> Types<'ast, 'a> {
    essential_trait(environment, "Render").expect("Render cannot be found")
}

fn essential_trait<'ast, 'a>(
    environment: &Environment<'ast, 'a>,
    name: &'static str,
) -> Option<Types<'ast, 'a>> {
    let std_essential = environment
        .find_module(STD_ESSENTIAL)
        .map(|scope_id| environment.get_scope(scope_id))?;
    std_essential
        .name_spaces
        .declared
        .get(name)
        .map(|declared| match declared {
            DeclaredElement::Trait(r#trait) => Types::Trait(*r#trait),
            _ => panic!("{name} is not declared as a trait"),
        })
}
