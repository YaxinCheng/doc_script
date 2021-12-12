mod declarations;
mod expressions;
mod field;
mod foundations;
mod import;
mod name;
mod parameter;
mod scoped_elements;
mod statements;
#[cfg(test)]
mod tests;

use crate::parser::{Node, NodeKind};

pub use declarations::*;
pub use expressions::*;
pub use field::*;
pub use foundations::*;
pub use import::*;
pub use name::*;
pub use parameter::*;
pub use scoped_elements::*;
pub use statements::*;

macro_rules! check_unpack {
    ($source: expr, $kind: pat) => {{
        crate::ast::debug_check!($source, Node::Internal { kind: $kind, .. });
        $source
            .children()
            .expect("Internal nodes always have children")
    }};
}

macro_rules! debug_check {
    ($source: expr, $pattern: pat) => {{
        debug_assert!(
            matches!($source, $pattern),
            "Wrong kind of node is passed to function: {:?}",
            $source
        );
    }};
}

pub(crate) use check_unpack;
pub(crate) use debug_check;

pub struct AbstractSyntaxTree<'a> {
    pub compilation_unit: CompilationUnit<'a>,
}

pub fn abstract_tree(parse_tree: crate::parser::ParseTree) -> AbstractSyntaxTree {
    AbstractSyntaxTree {
        compilation_unit: CompilationUnit::from(parse_tree.root),
    }
}
