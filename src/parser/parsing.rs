use super::models::{NodeKind, Production, Symbol};
use super::rules;
use super::{LiteralKind, Token, TokenKind};

pub type State = usize;
pub const START_STATE: State = 0;
pub const START_TOKEN: Token = Token {
    kind: TokenKind::ParsingStart,
    lexeme: "",
};
pub const END_TOKEN: Token = Token {
    kind: TokenKind::ParsingEnd,
    lexeme: "",
};

include!(concat!(env!("OUT_DIR"), "/action_table.rs"));
include!(concat!(env!("OUT_DIR"), "/symbols.rs"));

pub fn transit(state: State, symbol: Symbol) -> Option<State> {
    let symbol_index: usize = symbol_to_ord(symbol);
    let transitions: &[(usize, usize)] = TRANSITIONS[state];
    let index = transitions
        .binary_search_by_key(&symbol_index, |(symbol, _)| *symbol)
        .ok()?;
    Some(transitions[index].1)
}

pub fn reduce(state: State, terminal: Token) -> Option<Production> {
    let symbol_index: usize = symbol_to_ord(Symbol::Terminal(terminal));
    let reductions: &[(usize, usize)] = REDUCTIONS[state];
    let index = reductions
        .binary_search_by_key(&symbol_index, |(symbol, _)| *symbol)
        .ok()?;
    Some(rules::RULES[reductions[index].1])
}
