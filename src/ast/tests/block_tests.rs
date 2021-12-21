use crate::ast::{Block, Expression};
use crate::parser::{parse, NodeKind};
use crate::search::DepthFirst;
use crate::tokenizer::tokenize;

#[test]
fn test_statement_order_in_block() {
    let block = get_block(
        r#"
    const test = {
        3
        4
        5
    }
    "#,
    );
    let actual = block
        .statements
        .into_iter()
        .filter_map(|statement| statement.into_expression().ok())
        .filter_map(|expression| expression.into_literal().ok())
        .map(|(_kind, lexeme)| lexeme)
        .collect::<Vec<_>>();
    let expected = vec!["3", "4", "5"];
    assert_eq!(expected, actual)
}

#[test]
fn test_block_const_declaration_desugar() {
    let block = get_block(
        r#"
    const test = {
        const a = 3
        const b = a
    }
    "#,
    );
    let declaration_of_a = block.statements[0]
        .as_constant_declaration()
        .map(|constant| constant.name);
    assert_eq!(declaration_of_a, Some("a"));
    let block_for_b = block
        .statements
        .last()
        .and_then(|statement| statement.as_expression())
        .and_then(|expression| expression.as_block())
        .expect("Failed to get block for b");
    let declaration_of_b = block_for_b.statements[0]
        .as_constant_declaration()
        .map(|constant| constant.name);
    assert_eq!(declaration_of_b, Some("b"))
}

fn get_block(program: &str) -> Block {
    let parse_tree = parse(tokenize(program));
    DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::Block)),
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .map(Expression::from)
    .and_then(|expression| expression.into_block().ok())
    .expect("Cannot find block")
}
