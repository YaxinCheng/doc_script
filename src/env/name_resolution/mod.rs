mod resolution;
mod resolve_helper;
mod resolved;
#[cfg(test)]
mod tests;
pub mod types;

use super::Environment;
pub(in crate::env) use resolution::NameResolver;
pub use resolved::Resolved;
