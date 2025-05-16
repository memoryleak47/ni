use crate::*;

pub type FnId = usize;
pub type BlockId = usize;
pub type Node = usize;

pub type Block = Vec<Stmt>;

pub struct IR {
    pub fns: Map<FnId, Function>,
    pub main_fn: FnId,
}

pub struct Function {
    pub blocks: Map<BlockId, Block>,
    pub start_block: BlockId,
}

pub enum BinOpKind {
    Plus, Minus, Mul, Div, IntDiv, Mod, Pow,
    Lt, Le, Gt, Ge,
    IsEqual, IsNotEqual,
}

enum Stmt {
    Compute(Node, Expr), // create a new node with the value returned from the Expr.
    Store(/*table: */ Node, /*index: */ Node, Node), // store the value from the Node in the table `table` at index `index`.
    If(Node, /*then*/ BlockId, /*else*/ BlockId),
    FnCall(/*func: */ Node, /* arg: */ Node),
    Print(Node),
    Crash(String), // something bad happened.
    Return,
}

enum Expr {
    Index(/*table: */ Node, /*index: */ Node),

    Arg,
    NewTable, // equivalent to {}
    Function(FnId),
    BinOp(BinOpKind, Node, Node),
    Len(Node),
    Next(Node, Node),
    Type(Node),

    // literals
    Num(R64),
    Bool(bool),
    Nil,
    Str(String),
}
