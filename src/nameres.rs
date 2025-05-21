use crate::*;

pub enum VarPlace {
	Global, // unknown: check global module namespace and then builtins.
	// TODO add
	// Closured(/*how many scopes out?*/ usize),
	Local,
}

pub type NameResTable = Map</*ptr address of ASTExpr::Var(_)*/ usize, VarPlace>;

pub fn nameres(ast: &AST) -> NameResTable {
	let mut map = NameResTable::new();
	map
}
