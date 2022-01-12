mod address_hash;
mod checks;
mod construction;
mod declaration_resolution;
mod env_builder;
mod environment;
mod name_resolution;
pub mod scope;

pub use environment::{Environment, Resolved};
pub type EnvironmentBuilder<'ast, 'a> = env_builder::EnvironmentBuilder<'ast, 'a, 0>;
pub use name_resolution::TypedElement;
