use std::fs;

pub use std::collections::HashSet as Set;
pub use std::collections::HashMap as Map;

pub use noisy_float::prelude::{R64, Float};

mod ast;
pub use ast::*;

mod tokenize;
pub use tokenize::*;

mod assemble;
pub use assemble::*;

mod ir;
pub use ir::*;

fn main() {
	let filename = std::env::args().nth(1).expect("Missing CLI argument!");
	let contents = fs::read_to_string(filename).unwrap();
	let toks = tokenize(&contents);
	dbg!(&toks);
	let ast = assemble(&toks);
	dbg!(&ast);
}
