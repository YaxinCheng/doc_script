use super::models::{NodeKind, Production, Symbol};
use super::TokenKind::*;
use super::{LiteralKind, Token};

include!(concat!(env!("OUT_DIR"), "/rules.rs"));
