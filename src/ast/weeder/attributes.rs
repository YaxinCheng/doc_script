use crate::ast::{ConstantDeclaration, Expression};

pub fn weed(const_declaration: &ConstantDeclaration) {
    if let Expression::SelfRef(_) = const_declaration.value {
        panic!("Struct attributes cannot expose self")
    }
}

#[cfg(test)]
mod attributes_weeder_tests {
    use super::weed;
    use crate::ast::{ConstantDeclaration, Expression};

    #[test]
    #[should_panic]
    fn test_expose_self() {
        let constant_decl = ConstantDeclaration {
            name: "constant",
            value: Expression::SelfRef(None),
        };
        weed(&constant_decl)
    }

    #[test]
    fn test_no_self_exposed() {
        let constant_decl = ConstantDeclaration {
            name: "constant",
            value: Expression::Void,
        };
        weed(&constant_decl);
    }
}
