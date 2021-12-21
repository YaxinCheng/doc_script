mod checks;
mod construction;
mod declaration_resolution;
mod environment;
mod name_resolution;
pub mod scope;
// mod disambiguate;
mod address_hash;
mod env_builder;

pub use env_builder::EnvironmentBuilder;
pub use environment::Environment;

#[cfg(test)]
pub(in crate::env) fn construct_env<'ast, 'a>() -> Environment<'ast, 'a> {
    Environment::default()
}
