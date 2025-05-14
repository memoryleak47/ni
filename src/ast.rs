pub type AST = Vec<Stmt>;

pub enum Stmt {
	Assign(Expr, Expr),
	Def(/*fn name*/ String, /*args*/ Vec<String>, /*body*/ AST),
	If(Expr, AST, AST),
	While(AST),
	Break,
	Continue,
	Expr(Expr),
}

pub enum Expr {
	Var(String),
	ConstNum(f64),
	Add(Box<Expr>, Box<Expr>),
	FnCall(Box<Expr>, Vec<Expr>),
}
