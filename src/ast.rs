use crate::*;

pub type AST = Vec<ASTStatement>;

#[derive(Debug, Clone)]
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
    Try(AST),
    Except(AST),
    Raise(ASTExpr),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ScopeKind {
    Global,
    NonLocal,
}

#[derive(Debug, Clone)]
pub enum ASTExpr {
    Var(String),
    Str(String),
    Int(i64),
    Bool(bool),
    List(Vec<ASTExpr>),
    None,
    FnCall(Box<ASTExpr>, Vec<ASTExpr>),
    BinOp(BinOpKind, Box<ASTExpr>, Box<ASTExpr>),
    Attribute(Box<ASTExpr>, String),
}
