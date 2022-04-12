#![cfg(test)]

mod multi_file_tests;
mod single_file_tests;

use lazy_static::lazy_static;

// lock used to guarantee when tests run, only one thread can compile at a time
lazy_static! {
    pub static ref COMPILER_LOCK: std::sync::Mutex<()> = std::sync::Mutex::default();
}
