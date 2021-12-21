use super::hash;
use super::{type_resolver, Error, Types};
use crate::ast::StructDeclaration;
use crate::env::Environment;
use std::collections::HashSet;

hash!(StructDeclaration);

pub struct StructHierarchyChecker<'ast, 'a, 'env> {
    declaring: HashSet<&'ast StructDeclaration<'a>>,
    environment: &'env Environment<'ast, 'a>,
}

impl<'ast, 'a, 'env> StructHierarchyChecker<'ast, 'a, 'env> {
    pub fn with_environment(environment: &'env Environment<'ast, 'a>) -> Self {
        Self {
            declaring: HashSet::new(),
            environment,
        }
    }

    pub fn recursively_check(
        mut self,
        declaration: &'ast StructDeclaration<'a>,
        white_list: &mut HashSet<&'ast StructDeclaration<'a>>,
    ) -> Result<(), Error> {
        self._recursively_check(declaration, white_list)
    }

    fn _recursively_check(
        &mut self,
        declaration: &'ast StructDeclaration<'a>,
        white_list: &mut HashSet<&'ast StructDeclaration<'a>>,
    ) -> Result<(), Error> {
        if !self.declaring.insert(declaration) {
            Err(Error::StructCycleDependency(declaration.name.to_owned()))
        } else if white_list.contains(&declaration) {
            // already in whitelist
            Ok(())
        } else {
            for field in &declaration.fields {
                match type_resolver::resolve_type_name(self.environment, &field.field_type.0) {
                    Some(Types::Struct(struct_declaration)) => {
                        self._recursively_check(struct_declaration, white_list)?;
                    }
                    Some(_primitive_types) => (),
                    None => panic!("Name `{}` cannot be resolved", field.field_type.0),
                }
            }
            self.declaring.remove(&declaration);
            white_list.insert(declaration);
            Ok(())
        }
    }
}
