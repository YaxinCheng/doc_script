use super::{resolve_helper::ResolveHelper, Resolved};
use crate::ast::{Moniker, Name, StructDeclaration};
use crate::env::scope::Scoped;
use crate::env::Environment;

pub struct TypeLinker<'ast, 'a, 'env>(pub &'env mut Environment<'ast, 'a>);

type TypeDeclaration<'a> = StructDeclaration<'a>;

impl<'ast, 'a, 'env> TypeLinker<'ast, 'a, 'env> {
    pub fn link_types<I: IntoIterator<Item = &'ast Name<'a>>>(self, names: I) {
        for type_name in names {
            let type_declaration = self.link_type(type_name);
            if let Some(type_declaration) = type_declaration {
                self.0
                    .resolved_names
                    .insert(type_name.clone(), Resolved::Struct(type_declaration));
            } else if !Self::is_primitive_type(type_name) {
                if let Some(type_declaration) = self.link_type_in_module(type_name) {
                    self.0
                        .resolved_names
                        .insert(type_name.clone(), Resolved::Struct(type_declaration));
                } else {
                    panic!("Failed to resolve type name: `{}`", type_name)
                }
            }
        }
    }

    fn link_type(&self, name: &'ast Name<'a>) -> Option<&'ast TypeDeclaration<'a>> {
        ResolveHelper(self.0)
            .resolve(name.scope(), &name.moniker)
            .map(|(resolved, _)| match resolved {
                Resolved::Struct(struct_type) => struct_type,
                Resolved::InstanceAccess(_) => {
                    panic!("Type name `{}` resolved to field access", name)
                }
                Resolved::Field { .. } => panic!("Type name `{}` resolved to field", name),
                Resolved::Constant(_) => panic!("Type name `{}` resolved to constant", name),
                Resolved::Module(_) => panic!("Type name `{}` resolved to module", name),
            })
    }

    fn link_type_in_module(&self, name: &'ast Name<'a>) -> Option<&'ast TypeDeclaration<'a>> {
        let (resolved, not_resolved) =
            ResolveHelper(self.0).resolve_module_lead_name(name.scope(), &name.moniker);
        match (resolved, not_resolved.is_empty()) {
            (Resolved::Struct(struct_declaration), true) => Some(struct_declaration),
            _ => None,
        }
    }

    fn is_primitive_type(name: &'ast Name<'a>) -> bool {
        matches!(
            name.moniker,
            Moniker::Simple("Int" | "Float" | "String" | "Bool")
        )
    }
}
