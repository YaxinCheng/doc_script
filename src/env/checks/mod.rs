mod struct_hierarchy;
#[cfg(test)]
mod tests;
pub mod type_checking;

use super::address_hash::hash;
use type_checking::types::Types;

pub(in crate::env) use struct_hierarchy::StructHierarchyChecker;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Struct cycle dependency detected at {0}")]
    StructCycleDependency(String),
}
