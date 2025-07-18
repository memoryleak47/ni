pub use noisy_float::prelude::*;

use crate::*;

mod fmt;
pub use fmt::*;

mod tokenize;
pub use tokenize::*;

mod assemble;
pub use assemble::*;

mod exec;
pub use exec::*;

pub type Stmt = (Symbol, /*idx*/ usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOpKind {
    Plus, Minus, Mul, Div, Mod, Pow,
    Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node(pub Symbol);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Let(Node, Expr, /*visible*/ bool), // create a new node with the value returned from the Expr.
    Store(/*table: */ Node, /*index: */ Node, Node), // store the value from the Node in the table `table` at index `index`.
    Print(Node),
    Jmp(/*proc-id*/ Node),
    Panic(/*err msg or code*/ Node),
    Fail, // analysis tries to prove that this is never reached.
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Index(/*table: */ Node, /*index: */ Node),

    Root, // @
    NewTable, // equivalent to {}
    BinOp(BinOpKind, Node, Node),

    Input, // corresponds to pythons "input()" function applied to no args.

    // literals
    Symbol(Symbol), // $symbol
    Float(R64),
    Int(i64),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct Proc {
    pub stmts: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct IR {
    pub procs: Map<Symbol, Proc>,
    pub main_pid: Symbol,
}
