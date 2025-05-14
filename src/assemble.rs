use crate::*;

pub fn assemble(toks: &[Token]) -> AST {
	let (ast, toks) = assemble_ast(toks).unwrap();
	assert!(toks.is_empty());

	ast
}

fn assemble_ast(mut toks: &[Token]) -> Result<(AST, &[Token]), String> {
	let mut stmts = Vec::new();
	while let Ok((x, rst)) = assemble_stmt(toks) {
		toks = rst;
		stmts.push(x);
	}
	Ok((stmts, toks))
}

fn assemble_expr(toks: &[Token]) -> Result<(Expr, &[Token]), String> {
	assemble_atomic_expr(toks)
	// TODO calls
}

fn assemble_atomic_expr(toks: &[Token]) -> Result<(Expr, &[Token]), String> {
	match toks.get(0) {
		Some(Token::Ident(x)) => Ok((Expr::Var(x.clone()), &toks[1..])),
		_ => Err(String::new()),
	}
}

fn assemble_stmt(toks: &[Token]) -> Result<(Stmt, &[Token]), String> {
	or(assemble_stmt_base, or(assemble_def_stmt, assemble_expr_stmt))(toks)
}

fn assemble_stmt_base(toks: &[Token]) -> Result<(Stmt, &[Token]), String> {
	match toks.get(0) {
		Some(Token::Break) => Ok((Stmt::Break, &toks[1..])),
		Some(Token::Continue) => Ok((Stmt::Continue, &toks[1..])),
		_ => Err(String::new()),
	}
}

fn assemble_def_stmt(toks: &[Token]) -> Result<(Stmt, &[Token]), String> {
	Err(String::new())
}

fn assemble_expr_stmt(toks: &[Token]) -> Result<(Stmt, &[Token]), String> {
	let (expr, toks) = assemble_expr(toks)?;
	if toks[0] == Token::Equals {
		let (rhs, toks) = assemble_expr(&toks[1..])?;
		Ok((Stmt::Assign(expr, rhs), toks))
	} else {
		Ok((Stmt::Expr(expr), toks))
	}
}


trait Assembler<T>: for<'a> Fn(&[Token]) -> Result<(T, &[Token]), String> {}
impl<A, T> Assembler<T> for A where A: for<'a> Fn(&[Token]) -> Result<(T, &[Token]), String> {}

fn or<T>(a: impl Assembler<T>, b: impl Assembler<T>) -> impl Assembler<T> {
	move |toks| a(toks).or_else(|err| b(toks).map_err(|err2| format!("({err})|({err2})")))
}

fn chain<T1, T2, O>(a: impl Assembler<T1>, b: impl Assembler<T2>, f: impl Fn(T1, T2) -> O) -> impl Assembler<O> {
	move |toks| a(toks).and_then(|(x, toks)| b(toks).map(|(y, toks)| (f(x, y), toks)))
}
