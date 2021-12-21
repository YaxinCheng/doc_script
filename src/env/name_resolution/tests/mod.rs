mod const_field_hierarchy_tests;
mod test_chaining_method;
mod test_disambiguate;
mod test_name_resolution;
mod test_resolve_helper;
mod test_resolve_types;
mod test_type_linker;

use super::super::construct_env;

macro_rules! try_block {
    ($kind: ty, $block: expr) => {{
        let __try_block = || -> Option<$kind> { $block };
        __try_block()
    }};
}

pub(in crate::env::name_resolution::tests) use try_block;
