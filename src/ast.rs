pub type AST = Vec<Stmt>;

#[derive(Debug)]
pub enum Stmt {
	Assign(Expr, Expr),
	Def(/*fn name*/ String, /*args*/ Vec<String>, /*body*/ AST),
	If(Expr, AST),
	While(Expr, AST),
	Break,
	Continue,
	Expr(Expr),
}

#[derive(Debug)]
pub enum Expr {
	Var(String),
	ConstNum(f64),
	Add(Box<Expr>, Box<Expr>),
	FnCall(Box<Expr>, Vec<Expr>),
}
