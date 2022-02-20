use crate::ast::TraitDeclaration;
use crate::env::scope::DeclaredElement;
use crate::env::Environment;

const STD_ESSENTIAL: &[&str] = &["std", "essential"];

pub fn render<'ast, 'a>(environment: &Environment<'ast, 'a>) -> Option<&'ast TraitDeclaration<'a>> {
    essential_trait(environment, "Render")
}

fn essential_trait<'ast, 'a>(
    environment: &Environment<'ast, 'a>,
    name: &'static str,
) -> Option<&'ast TraitDeclaration<'a>> {
    let std_essential = environment
        .find_module(STD_ESSENTIAL)
        .map(|scope_id| environment.get_scope(scope_id))?;
    std_essential
        .name_spaces
        .declared
        .get(name)
        .and_then(|declared| match declared {
            DeclaredElement::Trait(r#trait) => Some(*r#trait),
            _ => None,
        })
}
