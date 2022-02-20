use crate::ast::weeder::attributes;
use crate::ast::{ConstantDeclaration, Expression, Statement};
use crate::env::scope::*;
use scope_macro::Scoped;
use std::collections::VecDeque;

#[derive(Scoped, Debug, Eq, PartialEq)]
pub struct Block<'a> {
    pub statements: Vec<Statement<'a>>,
    scope: Option<ScopeId>,
}

impl<'a> FromIterator<Statement<'a>> for Block<'a> {
    fn from_iter<T: IntoIterator<Item = Statement<'a>>>(iter: T) -> Self {
        let mut statement_stack = iter.into_iter().collect::<Vec<_>>();
        let mut statements = VecDeque::new();
        while let Some(statement) = statement_stack.pop() {
            match statement {
                expr_stmt @ Statement::Expression(_) => statements.push_front(expr_stmt),
                const_stmt @ Statement::ConstantDeclaration(_) => {
                    if !statements.is_empty() {
                        let block = Block {
                            statements: Vec::from(statements),
                            scope: None,
                        };
                        statements = VecDeque::new();
                        statements.push_front(Statement::Expression(Expression::Block(block)));
                    }
                    statements.push_front(const_stmt);
                }
            }
        }
        Self {
            statements: Vec::from(statements),
            scope: None,
        }
    }
}

#[cfg(test)]
impl<'a> From<Vec<Statement<'a>>> for Block<'a> {
    fn from(statements: Vec<Statement<'a>>) -> Self {
        Self {
            statements,
            scope: None,
        }
    }
}

#[derive(Default, Scoped, Debug, Eq, PartialEq)]
pub struct StructBody<'a> {
    pub attributes: Vec<ConstantDeclaration<'a>>,
    scope: Option<ScopeId>,
}

impl<'a> FromIterator<ConstantDeclaration<'a>> for StructBody<'a> {
    fn from_iter<T: IntoIterator<Item = ConstantDeclaration<'a>>>(iter: T) -> Self {
        let declarations = iter.into_iter().inspect(attributes::weed).collect();
        Self {
            attributes: declarations,
            scope: None,
        }
    }
}

#[cfg(test)]
impl<'a> From<Vec<ConstantDeclaration<'a>>> for StructBody<'a> {
    fn from(declarations: Vec<ConstantDeclaration<'a>>) -> Self {
        Self {
            attributes: declarations,
            scope: None,
        }
    }
}
