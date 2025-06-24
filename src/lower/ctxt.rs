use crate::lower::*;
use crate::*;

// able to push new statements to this current function.
pub(in crate::lower) struct FnCtxt {
    pub node_ctr: usize,
    pub current_fn: FnId,
    pub current_blk: BlockId,
    pub lowering: Option<FnLowerCtxt>, // set to None for builtin functions.
    pub singletons_node: Node,
    pub arg_node: Node,
}

pub(in crate::lower) struct Ctxt {
    pub stack: Vec<FnCtxt>,
    pub nameres_tab: NameResTable,
    pub ir: IR,
    pub builtin_fns: Map<String, FnId>,
}

impl Ctxt {
    pub fn f(&self) -> &FnCtxt {
        self.stack.last().unwrap()
    }

    pub fn f_mut(&mut self) -> &mut FnCtxt {
        self.stack.last_mut().unwrap()
    }

    pub fn push_compute(&mut self, expr: Expr) -> Node {
        let n = self.f().node_ctr;
        self.f_mut().node_ctr += 1;
        self.push_statement(Statement::Compute(n, expr));
        n
    }

    pub fn push_table(&mut self) -> Node {
        self.push_compute(Expr::NewTable)
    }

    pub fn push_index(&mut self, t: Node, k: Node) -> Node {
        self.push_compute(Expr::Index(t, k))
    }

    pub fn push_int(&mut self, i: i64) -> Node {
        self.push_compute(Expr::Int(i))
    }

    pub fn push_bool(&mut self, b: bool) -> Node {
        self.push_compute(Expr::Bool(b))
    }

    pub fn push_eq(&mut self, a: Node, b: Node) -> Node {
        self.push_compute(Expr::BinOp(BinOpKind::IsEqual, a, b))
    }

    pub fn push_undef(&mut self) -> Node {
        self.push_compute(Expr::Undef)
    }

    pub fn push_return(&mut self) {
        self.push_statement(Statement::Return)
    }

    pub fn push_str(&mut self, s: &str) -> Node {
        self.push_compute(Expr::Str(s.to_string()))
    }

    pub fn push_store(&mut self, t: Node, k: Node, v: Node) {
        self.push_statement(Statement::Store(t, k, v));
    }

    pub fn push_store_str(&mut self, t: Node, k: &str, v: Node) {
        let k = self.push_str(k);
        self.push_statement(Statement::Store(t, k, v));
    }

    pub fn push_index_str(&mut self, t: Node, k: &str) -> Node {
        let k = self.push_str(k);
        self.push_compute(Expr::Index(t, k))
    }

    pub fn push_store_int(&mut self, t: Node, k: usize, v: Node) {
        let k = self.push_int(k as i64);
        self.push_statement(Statement::Store(t, k, v));
    }

    pub fn push_index_int(&mut self, t: Node, k: usize) -> Node {
        let k = self.push_int(k as i64);
        self.push_compute(Expr::Index(t, k))
    }

    pub fn push_statement(&mut self, stmt: Statement) {
        let current_fn = self.f().current_fn;
        let current_blk = self.f().current_blk;
        self.ir
            .fns
            .get_mut(&current_fn)
            .unwrap()
            .blocks
            .get_mut(&current_blk)
            .unwrap()
            .push(stmt);
    }

    pub fn push_print(&mut self, x: &str) {
        let x = self.push_str(x);
        self.push_statement(Statement::Print(x));
    }

    pub fn push_goto(&mut self, b: BlockId) {
        let true_ = self.push_bool(true);
        self.push_statement(Statement::If(true_, b, b));
    }

    pub fn push_if(&mut self, cond: Node, b1: BlockId, b2: BlockId) {
        self.push_statement(Statement::If(cond, b1, b2));
    }

    pub fn alloc_blk(&mut self) -> BlockId {
        let current_fn = self.f().current_fn;
        let f = self.ir.fns.get_mut(&current_fn).unwrap();
        let n = f.blocks.len();
        f.blocks.insert(n, Vec::new());
        n
    }

    pub fn focus_blk(&mut self, b: BlockId) {
        self.f_mut().current_blk = b;
    }
}

// Generic Ctxt functions that go beyond general IR construction, and are tainted by our lowering procedure:
impl Ctxt {
    pub fn get_singleton(&mut self, v: &str) -> Node {
        self.push_index_str(self.f().singletons_node, v)
    }

    pub fn push_builtin(&mut self, s: &str) -> Node {
        self.push_compute(Expr::Function(self.builtin_fns[s]))
    }
}

pub fn new_fn(ctxt: &mut Ctxt, f: impl FnOnce(&mut Ctxt)) -> FnId {
    new_fn_general(false, ctxt, f)
}

pub fn new_fn_general(main: bool, ctxt: &mut Ctxt, f: impl FnOnce(&mut Ctxt)) -> FnId {
    let n = ctxt.ir.fns.len();
    let mut blocks: Map<_, _> = Default::default();

    // general function setup:
    if main {
        blocks.insert(0, vec![
            Statement::Compute(0, Expr::Undef),
            Statement::Compute(1, Expr::Bool(true)),
            Statement::Compute(2, Expr::NewTable),
            Statement::If(1, 1, 1),
        ]);
    } else {
        blocks.insert(0, vec![
            Statement::Compute(0, Expr::Arg),
            Statement::Compute(1, Expr::Str("singletons".to_string())),
            Statement::Compute(2, Expr::Index(0, 1)),
            Statement::Compute(3, Expr::Bool(true)),
            Statement::If(3, 1, 1),
        ]);
    }
    blocks.insert(1, Vec::new());

    ctxt.ir.fns.insert(
        n,
        Function {
            blocks,
            start_block: 0,
        },
    );

    ctxt.stack.push(FnCtxt {
        node_ctr: 3,
        current_fn: n,
        current_blk: 1,
        arg_node: 0,
        singletons_node: 2,
        lowering: None,
    });

    f(ctxt);

    ctxt.stack.pop();

    n
}
