mod resolve_types_tests;
mod struct_hierarchy_tests;
mod test_type_check_chaining_method;
mod render_tests;

macro_rules! try_block {
    ($kind: ty, $block: expr) => {{
        let __try_block = || -> Option<$kind> { $block };
        __try_block().unwrap()
    }};
}

pub(in crate::env::checks::tests) use try_block;
