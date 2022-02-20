pub(in crate::env::checks) mod assignable_checker;
pub(in crate::env::checks) mod essential_trait;
mod render_impl_checker;
mod struct_init_checker;
mod type_checker;
pub mod type_resolver;
pub mod types;

pub(in crate::env) use type_checker::TypeChecker;
