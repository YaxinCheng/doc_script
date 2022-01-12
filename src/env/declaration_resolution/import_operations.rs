use crate::ast::{AbstractSyntaxTree, Declaration, Import};
use crate::env::scope::{DeclaredElement, Scope, ScopeId};
use crate::env::Environment;

pub(in crate::env::declaration_resolution) struct Importer<'ast, 'a, 'env>(
    pub &'env mut Environment<'ast, 'a>,
);

impl<'ast, 'a, 'env> Importer<'ast, 'a, 'env> {
    pub fn import_from(
        &mut self,
        syntax_trees: &'ast [AbstractSyntaxTree<'a>],
        module_paths: &[Vec<&'a str>],
    ) {
        let mut importing_elements = vec![];
        // process import declarations, then do import separately
        for (syntax_tree, module_path) in syntax_trees.iter().zip(module_paths.iter()) {
            let target_scope_id = self
                .0
                .find_module(module_path)
                .unwrap_or_else(|| panic!("Cannot find module: {}", module_path.join(".")));
            let elements = Self::find_imports(syntax_tree)
                .flat_map(|import| self.process_import(import).into_iter())
                .collect::<Vec<_>>();
            importing_elements.push((target_scope_id, elements));
        }
        // actual imports
        for (target_scope_id, elements) in importing_elements {
            let target_scope = self.0.get_scope_mut(target_scope_id);
            for element in elements {
                element.import_to(target_scope)
            }
        }
    }

    fn find_imports(
        syntax_tree: &'ast AbstractSyntaxTree<'a>,
    ) -> impl Iterator<Item = &'ast Import<'a>> {
        syntax_tree
            .compilation_unit
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Import(import) => Some(import),
                _ => None,
            })
    }

    fn process_import(&mut self, import: &'ast Import<'a>) -> Vec<Importing<'ast, 'a>> {
        match import {
            Import::Single(name) => vec![self.process_single_import(name)],
            Import::Wildcard(module) => vec![self.process_wildcard_import(module)],
            Import::Multiple { prefix, suffices } => suffices
                .iter()
                .map(|suffix| self.process_multiple_import(prefix, suffix))
                .collect(),
        }
    }

    fn process_wildcard_import(&mut self, module: &'ast [&'a str]) -> Importing<'ast, 'a> {
        let source_scope_id = self
            .0
            .find_module(module)
            .unwrap_or_else(|| panic!("Module name ({}) is invalid", module.join(".")));
        Importing::Wildcard(source_scope_id, module)
    }

    fn process_multiple_import(
        &mut self,
        prefix: &'ast [&'a str],
        suffix: &'ast [&'a str],
    ) -> Importing<'ast, 'a> {
        self.process_single_import(&[prefix, suffix].concat())
    }

    fn process_single_import(&mut self, import: &[&'a str]) -> Importing<'ast, 'a> {
        debug_assert!(!import.is_empty(), "Cannot import empty module");
        let (last_element, module_path) =
            import.split_last().unwrap_or((import.last().unwrap(), &[]));
        let scope = self
            .0
            .find_module(module_path)
            .map(|scope_id| self.0.get_scope(scope_id))
            .unwrap_or_else(|| panic!("Failed to resolve module: {}", module_path.join(".")));
        if let Some(declared) = scope.name_spaces.declared.get(last_element) {
            Importing::ExpressionOrStruct(*declared, last_element)
        } else if let Some(&scope_id) = scope.name_spaces.modules.get(last_element) {
            Importing::Module(scope_id, last_element)
        } else {
            panic!("Unknown type for {}", import.join("."))
        }
    }
}

enum Importing<'ast, 'a> {
    Wildcard(ScopeId, &'ast [&'a str]),
    Module(ScopeId, &'a str),
    ExpressionOrStruct(DeclaredElement<'ast, 'a>, &'a str),
}

impl<'ast, 'a> Importing<'ast, 'a> {
    pub fn import_to(self, target_scope: &mut Scope<'ast, 'a>) {
        use Importing::*;
        match self {
            Wildcard(scope_id, name) => Self::import_wildcard(scope_id, name, target_scope),
            Module(scope_id, name) => Self::import_module(scope_id, name, target_scope),
            ExpressionOrStruct(declared, name) => {
                Self::import_expression(declared, name, target_scope)
            }
        }
    }

    fn import_wildcard(
        scope_id: ScopeId,
        name: &'ast [&'a str],
        target_scope: &mut Scope<'ast, 'a>,
    ) {
        if !target_scope.name_spaces.wildcard_imports.insert(scope_id) {
            panic!("Module `{}` has already being imported", name.join("."))
        }
    }

    fn import_module(scope_id: ScopeId, module_name: &'a str, target_scope: &mut Scope<'ast, 'a>) {
        let existed_module = target_scope
            .name_spaces
            .modules
            .insert(module_name, scope_id);
        assert!(
            existed_module.is_none(),
            "Module `{}` has already being imported",
            module_name
        )
    }

    // constant or struct import can be shadowed
    fn import_expression(
        element: DeclaredElement<'ast, 'a>,
        name: &'a str,
        target_scope: &mut Scope<'ast, 'a>,
    ) {
        target_scope
            .name_spaces
            .declared
            .entry(name)
            .or_insert(element);
    }
}
