mod struct_hierarchy;
#[cfg(test)]
mod struct_hierarchy_tests;

use super::address_hash::hash;
use super::name_resolution::type_resolver;
use super::name_resolution::types::Types;

pub(in crate::env) use struct_hierarchy::StructHierarchyChecker;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Struct cycle dependency detected at {0}")]
    StructCycleDependency(String),
}
