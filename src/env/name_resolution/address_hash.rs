macro_rules! hash {
    ($type: ident) => {
        impl<'ast, 'a> std::hash::Hash for &'ast $type<'a> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                state.write_usize((self as *const _) as usize)
            }
        }
    };
}

use crate::ast::{Expression, Field};
hash!(Expression);
hash!(Field);
