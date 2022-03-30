#![cfg(test)]
use lazy_static::lazy_static;

mod formula_suppress;
mod multi_file_tests;
mod single_files_tests;

pub use formula_suppress::FormulaSuppress;

// lock used to guarantee when tests run, only one thread can compile at a time
lazy_static! {
    static ref COMPILER_LOCK: std::sync::Mutex<()> = std::sync::Mutex::default();
}
