use std::fs;

pub use std::collections::HashSet as Set;
pub use std::collections::HashMap as Map;

mod ast;
pub use ast::*;

mod tokenize;
pub use tokenize::*;

fn main() {
	let filename = std::env::args().nth(1).expect("Missing CLI argument!");
	let contents = fs::read_to_string(filename).unwrap();
	let toks = tokenize(&contents);
	dbg!(&toks);
}
