macro_rules! hash {
    ($type: ident) => {
        impl<'ast, 'a> std::hash::Hash for $type<'a> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                (self as *const $type).hash(state)
            }
        }
    };
}

pub(in crate::env) use hash;
