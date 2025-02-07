// src/main.rs
//! Entry point for the LAM project.

use lam::machine::core::ping;

fn main() {
    println!("{}", ping());
}