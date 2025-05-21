use crate::*;

pub type AST = Vec<ASTStatement>;

#[derive(Debug)]
pub enum ASTStatement {
    Assign(ASTExpr, ASTExpr),
    Def(
        /*fn name*/ String,
        /*args*/ Vec<String>,
        /*body*/ AST,
    ),
    Class(
        /*class name*/ String,
        /*superclasses*/ Vec<String>,
        /*body*/ AST,
    ),
    If(ASTExpr, AST),
    While(ASTExpr, AST),
    Break,
    Continue,
    Return(Option<ASTExpr>),
    Expr(ASTExpr),
    Pass,
    Scope(ScopeKind, Vec<String>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ScopeKind {
    Global,
    NonLocal,
}

#[derive(Debug)]
pub enum ASTExpr {
    Var(String),
    Str(String),
    Int(i64),
    Bool(bool),
    None,
    FnCall(Box<ASTExpr>, Vec<ASTExpr>),
    BinOp(BinOpKind, Box<ASTExpr>, Box<ASTExpr>),
}
