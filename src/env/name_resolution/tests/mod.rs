mod test_disambiguate;
mod test_resolve_helper;
mod test_resolve_types;

use super::super::construct_env;

macro_rules! try_block {
    ($kind: ty, $block: expr) => {{
        let __try_block = || -> Option<$kind> { $block };
        __try_block()
    }};
}

pub(in crate::env::name_resolution::tests) use try_block;
