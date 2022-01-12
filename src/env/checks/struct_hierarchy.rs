use super::hash;
use super::{type_checking::type_resolver, Error, Types};
use crate::ast::{AbstractSyntaxTree, Declaration, StructDeclaration};
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

    pub fn check(&mut self, syntax_trees: &'ast [AbstractSyntaxTree<'a>]) {
        let mut white_list = HashSet::new();
        for syntax_tree in syntax_trees {
            syntax_tree
                .compilation_unit
                .declarations
                .iter()
                .filter_map(|declaration| match declaration {
                    Declaration::Struct(struct_declaration) => Some(struct_declaration),
                    _ => None,
                })
                .for_each(|struct_declaration| {
                    self.recursively_check(struct_declaration, &mut white_list)
                        .expect("Cycle reference found in struct declaration")
                })
        }
    }

    fn recursively_check(
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
                        self.recursively_check(struct_declaration, white_list)?;
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

#[cfg(test)]
impl<'ast, 'a, 'env> StructHierarchyChecker<'ast, 'a, 'env> {
    pub fn test_recursively_check(
        &mut self,
        declaration: &'ast StructDeclaration<'a>,
        white_list: &mut HashSet<&'ast StructDeclaration<'a>>,
    ) -> Result<(), Error> {
        self.recursively_check(declaration, white_list)
    }
}
