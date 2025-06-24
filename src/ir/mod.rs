pub use noisy_float::prelude::*;

use crate::*;

mod fmt;
pub use fmt::*;

mod exec;
pub use exec::*;

pub type Stmt = (ProcId, /*idx*/ usize);

// the same as ast::BinOpKind but without And & Or.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOpKind {
    Plus, Minus, Mul, Div, Mod, Pow,
    Lt, Le, Gt, Ge,
    IsEqual, IsNotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node(pub Symbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcId(pub Symbol);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Let(Node, Expr), // create a new node with the value returned from the Expr.
    Store(/*table: */ Node, /*index: */ Node, Node), // store the value from the Node in the table `table` at index `index`.
    Print(Node),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Index(/*table: */ Node, /*index: */ Node),

    Root, // @
    NewTable, // equivalent to {}
    Proc(ProcId),
    BinOp(BinOpKind, Node, Node),

    // literals
    Float(R64),
    Int(i64),
    Bool(bool),
    Undef, // everything is initially Undef.
    Str(String),
}

#[derive(Debug, Clone)]
pub struct Procedure {
    pub stmts: Vec<Statement>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Jmp(/*proc-id*/ Node),
    Exit(/*err msg or code*/ Node),
}

#[derive(Debug, Clone)]
pub struct IR {
    pub procs: Map<ProcId, Procedure>,
    pub main_pid: ProcId,
}
