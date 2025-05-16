use crate::*;

pub type AST = Vec<ASTStatement>;

#[derive(Debug)]
pub enum ASTStatement {
	Assign(ASTExpr, ASTExpr),
	Def(/*fn name*/ String, /*args*/ Vec<String>, /*body*/ AST),
	Class(/*class name*/ String, /*superclasses*/ Vec<String>, /*body*/ AST),
	If(ASTExpr, AST),
	While(ASTExpr, AST),
	Break,
	Continue,
	Return,
	Expr(ASTExpr),
}

#[derive(Debug)]
pub enum ASTExpr {
	Var(String),
	Str(String),
	Int(i64),
	FnCall(Box<ASTExpr>, Vec<ASTExpr>),
	BinOp(BinOpKind, Box<ASTExpr>, Box<ASTExpr>)
}
