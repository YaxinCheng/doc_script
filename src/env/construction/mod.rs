mod module_operations;
mod scope_operations;
mod subdivide_struct_init;
#[cfg(test)]
mod tests;

use super::scope::*;
use super::Environment;
pub(in crate::env) use module_operations::ModuleAdder;
pub(in crate::env) use scope_operations::ScopeGenerator;
