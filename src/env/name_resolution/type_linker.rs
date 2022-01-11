use super::{resolve_helper::ResolveHelper, Resolved};
use crate::ast::{Moniker, Name};
use crate::env::scope::Scoped;
use crate::env::Environment;

pub struct TypeLinker<'ast, 'a, 'env>(pub &'env mut Environment<'ast, 'a>);

impl<'ast, 'a, 'env> TypeLinker<'ast, 'a, 'env> {
    pub fn link_types<I: IntoIterator<Item = &'ast Name<'a>>>(self, names: I) {
        for type_name in names {
            if let Some(type_declaration) = self.link_type(type_name) {
                self.0
                    .resolved_names
                    .insert(type_name.clone(), type_declaration);
            } else if !Self::is_primitive_type(type_name) {
                if let Some(type_declaration) = self.link_type_in_module(type_name) {
                    self.0
                        .resolved_names
                        .insert(type_name.clone(), type_declaration);
                } else {
                    panic!("Failed to resolve type name: `{}`", type_name)
                }
            }
        }
    }

    fn link_type(&self, name: &'ast Name<'a>) -> Option<Resolved<'ast, 'a>> {
        match &name.moniker {
            Moniker::Simple(simple_name) => {
                ResolveHelper(self.0).resolve(name.scope(), simple_name)
            }
            Moniker::Qualified(_) => Some(ResolveHelper(self.0).disambiguate(name)),
        }
        .map(|resolved| match resolved {
            Resolved::Struct(_) | Resolved::Trait(_) => resolved,
            Resolved::InstanceAccess(_, _) => {
                panic!("Type name `{}` resolved to field access", name)
            }
            Resolved::Constant(_) => panic!("Type name `{}` resolved to constant", name),
            Resolved::Module(_) => panic!("Type name `{}` resolved to module", name),
        })
    }

    fn link_type_in_module(&self, name: &'ast Name<'a>) -> Option<Resolved<'ast, 'a>> {
        let resolved = ResolveHelper(self.0).disambiguate(name);
        match resolved {
            Resolved::Struct(_) => Some(resolved),
            _ => None,
        }
    }

    fn is_primitive_type(name: &'ast Name<'a>) -> bool {
        let is_primitive_name =
            |name: &str| matches!(name, "Int" | "Float" | "String" | "Bool" | "()");
        match &name.moniker {
            Moniker::Simple(name) => is_primitive_name(name),
            Moniker::Qualified(full_name) => {
                full_name.len() == 2 && full_name[0] == "std" && is_primitive_name(full_name[1])
            }
        }
    }
}
