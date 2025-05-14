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

// (t1, t2, t3, ...)
fn assemble_paren_list<T>(sub: impl Assembler<T>) -> impl Assembler<Vec<T>> {
	move |toks| {
		if toks[0] != Token::LParen { return Err(String::new()); }
		if toks[1] == Token::RParen { return Ok((Vec::new(), &toks[2..])); }
		let toks = &toks[1..];

		let (first, mut toks) = sub(toks)?;
		let mut children = vec![first];

		while toks[0] == Token::Comma {
			toks = &toks[1..];
			let (c, rst) = sub(toks)?;
			toks = rst;
			children.push(c);
		}
		if toks[0] != Token::RParen { return Err(String::new()) };
		let toks = &toks[1..];
		Ok((children, toks))
	}
}

fn assemble_expr(toks: &[Token]) -> Result<(Expr, &[Token]), String> {
	let (expr, toks) = assemble_atomic_expr(toks)?;
	if let Some(Token::LParen) = &toks.get(0) {
		let (children, toks) = assemble_paren_list(assemble_expr)(toks)?;
		Ok((Expr::FnCall(Box::new(expr), children), toks))
	} else { Ok((expr, toks)) }
}

fn assemble_atomic_expr(toks: &[Token]) -> Result<(Expr, &[Token]), String> {
	match toks.get(0) {
		Some(Token::Ident(x)) => Ok((Expr::Var(x.clone()), &toks[1..])),
		_ => Err(String::new()),
	}
}

fn assemble_stmt(toks: &[Token]) -> Result<(Stmt, &[Token]), String> {
	or(assemble_stmt_base, or(assemble_def_stmt, or(assemble_expr_stmt, assemble_branch_stmt)))(toks)
}

fn assemble_branch_stmt(toks: &[Token]) -> Result<(Stmt, &[Token]), String> {
	let f = match toks.get(0) {
		Some(Token::If) => Stmt::If,
		Some(Token::While) => Stmt::While,
		_ => return Err(String::new()),
	};
	let toks = &toks[1..];
	let (expr, toks) = assemble_expr(toks)?;
	let (body, toks) = assemble_indented_ast(toks)?;
	Ok((f(expr, body), toks))
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
	if let Some(Token::Equals) = toks.get(0) {
		let (rhs, toks) = assemble_expr(&toks[1..])?;
		Ok((Stmt::Assign(expr, rhs), toks))
	} else {
		Ok((Stmt::Expr(expr), toks))
	}
}

fn assemble_indented_ast(toks: &[Token]) -> Result<(AST, &[Token]), String> {
	let Some(Token::Colon) = toks.get(0) else { return Err(String::new()) };
	let toks = &toks[1..];
	let Some(Token::Indent) = toks.get(0) else { return Err(String::new()) };
	let toks = &toks[1..];

	let (body, toks) = assemble_ast(toks)?;

	let Some(Token::Unindent) = toks.get(0) else { return Err(String::new()) };
	let toks = &toks[1..];

	Ok((body, toks))
}


trait Assembler<T>: for<'a> Fn(&[Token]) -> Result<(T, &[Token]), String> {}
impl<A, T> Assembler<T> for A where A: for<'a> Fn(&[Token]) -> Result<(T, &[Token]), String> {}

fn or<T>(a: impl Assembler<T>, b: impl Assembler<T>) -> impl Assembler<T> {
	move |toks| a(toks).or_else(|err| b(toks).map_err(|err2| format!("({err})|({err2})")))
}

fn chain<T1, T2, O>(a: impl Assembler<T1>, b: impl Assembler<T2>, f: impl Fn(T1, T2) -> O) -> impl Assembler<O> {
	move |toks| a(toks).and_then(|(x, toks)| b(toks).map(|(y, toks)| (f(x, y), toks)))
}
