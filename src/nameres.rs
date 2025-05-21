use crate::*;

pub enum VarPlace {
	Global, // unknown: check global module namespace and then builtins.
	// TODO add
	// Closured(/*how many scopes out?*/ usize),
	Local,
}

pub type NameResTable = Map<(/*ptr address of ASTStatement::Def(_)*/ *const ASTStatement, String), VarPlace>;

pub fn nameres(ast: &AST) -> NameResTable {
	let mut nrt = NameResTable::new();
	iter(ast, &mut nrt, 0 as _);
	nrt

}


fn iter(ast: &AST, nrt: &mut NameResTable, current_fn_ptr: *const ASTStatement) {
	for stmt in ast {
		match stmt {
			ASTStatement::Assign(ASTExpr::Var(v), _) => {
				nrt.insert((current_fn_ptr, v.to_string()), VarPlace::Local);
			},
			ASTStatement::Assign(..) => todo!(),
			ASTStatement::Def(_name, _args, body) => {
				iter(body, nrt, stmt as _);
			},
			ASTStatement::Class(..) => todo!(),
			ASTStatement::If(_, body) | ASTStatement::While(_, body) => {
				iter(body, nrt, current_fn_ptr);
			},
			ASTStatement::Break
			| ASTStatement::Continue
			| ASTStatement::Return(_)
			| ASTStatement::Expr(_)
			| ASTStatement::Pass => {},
		}
	}
}
