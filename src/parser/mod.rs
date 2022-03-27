mod models;
mod parsing;
mod rules;

use super::tokenizer::{LiteralKind, Token, TokenKind};
use crate::iterating::Iterating;
pub use models::ParseTree;
use models::Symbol;
pub use models::{Node, NodeKind};
use parsing::State;

pub fn parse<'a>(tokens: impl Iterator<Item = Token<'a>>) -> models::ParseTree<'a> {
    let mut state_stack: Vec<State> =
        vec![
            parsing::transit(parsing::START_STATE, Symbol::Terminal(parsing::START_TOKEN))
                .expect("Unable to start"),
        ];
    let mut node_stack: Vec<Node> = vec![Node::Leaf(parsing::START_TOKEN)];
    let top = |stack: &[_]| stack.last().copied().expect("Empty stack");
    let mut tokens = tokens.chain(std::iter::once(parsing::END_TOKEN)).peekable();
    while let Some(token) = tokens.next() {
        let normalized_tokens = skip_or_insert_new_lines(token, tokens.peek());
        for token in normalized_tokens {
            while let Some(production) = parsing::reduce(top(&state_stack), token) {
                let new_stack_size = node_stack.len() - production.rhs.len();
                let children = node_stack.drain(new_stack_size..).collect::<Vec<_>>();
                state_stack.truncate(new_stack_size);

                node_stack.push(Node::Internal {
                    kind: production.lhs,
                    children,
                });
                state_stack.push(
                    parsing::transit(top(&state_stack), Symbol::NonTerminal(production.lhs))
                        .unwrap_or_else(|| {
                            panic!("Unable to transit. node_stack={:?}", node_stack)
                        }),
                );
            }
            node_stack.push(Node::Leaf(token));
            state_stack.push(
                parsing::transit(top(&state_stack), Symbol::Terminal(token)).unwrap_or_else(|| {
                    panic!(
                        "Parsing error at token: {:?}. Stack: {:?}",
                        token, state_stack
                    )
                }),
            );
        }
    }
    node_stack.pop();
    ParseTree::from(node_stack.pop().expect("node_stack is empty"))
}

/// Skip or insert new line based on the current and next token.
/// When it skips, it returns an empty iterator
/// When it inserts, it return the an iterator with current token followed by a new line token
/// Otherwise, it returns an iterator containing the current token
///
/// condition to skip:
///     * when the current token is new line and the next token is . or ,
/// condition to add:
///     * when the current token does not suppress new line and the next token is curly closing bracket
fn skip_or_insert_new_lines<'a>(
    token: Token<'a>,
    next_token: Option<&Token>,
) -> impl Iterator<Item = Token<'a>> {
    match (token, next_token) {
        (
            Token {
                kind: TokenKind::NewLine,
                lexeme: _,
            },
            Some(Token {
                kind: TokenKind::Separator,
                lexeme: "." | ",",
            }),
        ) => Iterating::empty(),
        (
            token,
            Some(Token {
                kind: TokenKind::Separator,
                lexeme: "}",
            }),
        ) if !token.suppress_new_line() => Iterating::twice(token, NEW_LINE_TOKEN),
        _ => Iterating::once(token),
    }
}

const NEW_LINE_TOKEN: Token<'static> = Token {
    kind: TokenKind::NewLine,
    lexeme: "\n",
};

#[cfg(test)]
mod parse_tests {
    use super::models::NodeKind;
    use super::{Node, Token};

    #[test]
    fn test_const_declaration() {
        let text = "const i = 3\n";
        let parse_tree = super::parse(tokenize(text));
        let actual = first_child(&parse_tree.root, 3)
            .and_then(Node::kind)
            .expect("None obtained");
        let expected = NodeKind::ConstantDeclarationStatement;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_empty_struct_init() {
        let text = "const s = View(size: 5) { }\n";
        let parse_tree = super::parse(tokenize(text));
        let expected = NodeKind::ConstantDeclarationStatement;
        let actual = first_child(&parse_tree.root, 3)
            .and_then(Node::kind)
            .expect("None obtained");
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_struct_init_with_content() {
        let text = "const s = View(size: 5) { Text(\"abc\") }\n";
        let parse_tree = super::parse(tokenize(text));
        let expected = NodeKind::ConstantDeclarationStatement;
        let actual = first_child(&parse_tree.root, 3)
            .and_then(Node::kind)
            .expect("None obtained");
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_struct_init_with_multiple_content() {
        let text = "const s = View(size: 5) { Text(\"abc\")\n Text(content: \"another\")\n }\n";
        let parse_tree = super::parse(tokenize(text));
        let expected = NodeKind::ConstantDeclarationStatement;
        let actual = first_child(&parse_tree.root, 3)
            .and_then(Node::kind)
            .expect("None obtained");
        assert_eq!(actual, expected)
    }

    fn tokenize(text: &str) -> impl Iterator<Item = Token> {
        crate::tokenizer::tokenize(text)
    }

    fn first_child<'a, 'tree>(node: &'tree Node<'a>, levels: usize) -> Option<&'tree Node<'a>> {
        let mut node = node;
        for _ in 0..levels {
            node = match node {
                Node::Internal { kind: _, children } => children.first()?,
                _ => None?,
            };
        }
        Some(node)
    }
}
