// lowering to this layer does the following things:
// - decompose closures into tuples of functions and data
// - decompose generators into blocks
// - decompose exceptions into an exception handler linked-list stack, which are just handler functions

// questions:
// - do I have blocks & functions, or just one of them?

pub type FnId = usize;
pub type Blockid = usize;
pub type Node = usize;

pub type Body = Vec<Stmt>;

struct IR {
	fns: Vec<FnDef>,
	main: FnId,
}

struct FnDef {
	args: u32,
	body: Body,
}

enum Stmt {
	Compute(Node, Expr),
}

enum Expr {
	FnCall(Node, Node),
}
