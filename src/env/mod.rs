mod checks;
mod construction;
mod declaration_resolution;
mod environment;
mod name_resolution;
pub mod scope;
// mod disambiguate;
mod address_hash;
mod env_builder;

pub use environment::Environment;
pub type EnvironmentBuilder<'ast, 'a> = env_builder::EnvironmentBuilder<'ast, 'a, 0>;

#[cfg(test)]
pub(in crate::env) fn construct_env<'ast, 'a>() -> Environment<'ast, 'a> {
    Environment::default()
}
