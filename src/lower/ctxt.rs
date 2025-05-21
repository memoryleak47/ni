use crate::lower::*;

pub(in crate::lower) struct FnCtxt {
    pub node_ctr: usize,
    pub current_fn: FnId,
    pub current_blk: BlockId,
    pub loop_stack: Vec<(/*break*/ BlockId, /*continue*/ BlockId)>,
    pub namespace_node: Node,
    pub global_node: Node,
    pub singletons_node: Node,
    pub ast_ptr: *const ASTStatement, // the original def stmt
}


pub(in crate::lower) struct Ctxt {
    pub stack: Vec<FnCtxt>,
    pub nameres_tab: NameResTable,
    pub ir: IR,
    pub builtin_fns: HashMap<String, FnId>,
}

impl FnCtxt {
    pub fn new(f: FnId, ast_ptr: *const ASTStatement) -> Self {
        Self {
            node_ctr: 10,
            current_fn: f,
            current_blk: 0,
            loop_stack: Vec::new(),
            namespace_node: 0,
            global_node: 3,
            singletons_node: 5,
            ast_ptr,
        }
    }
}

impl Ctxt {
    pub fn f(&self) -> &FnCtxt {
        self.stack.last().unwrap()
    }

    pub fn f_mut(&mut self) -> &mut FnCtxt {
        self.stack.last_mut().unwrap()
    }

    pub fn new(ast: &AST) -> Self {
        let nameres_tab = nameres(ast);

        let mut fns: HashMap<_, _> = Default::default();
        let mut blocks: HashMap<_, _> = Default::default();
        blocks.insert(0, vec![Statement::Compute(0, Expr::NewTable)]);
        let main_fn = Function {
            blocks,
            start_block: 0,
        };
        fns.insert(0, main_fn);

        let mut fn_ctxt = FnCtxt::new(0, 0 as _);
        fn_ctxt.global_node = 0; // for the outer function, the global scope is actually it's local scope.
        Ctxt {
            stack: vec![fn_ctxt],
            nameres_tab,
            ir: IR { main_fn: 0, fns },
            builtin_fns: Default::default(),
        }
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

    pub fn push_none(&mut self) -> Node {
        self.push_compute(Expr::None)
    }

    pub fn push_arg(&mut self) -> Node {
        self.push_compute(Expr::Arg)
    }

    pub fn push_return(&mut self) {
        self.push_statement(Statement::Return)
    }

    pub fn push_builtin(&mut self, s: &str) -> Node {
        self.push_compute(Expr::Function(self.builtin_fns[s]))
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

    pub fn push_goto(&mut self, b: BlockId) {
        let true_ = lower_expr(&ASTExpr::Bool(true), self);
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

pub fn new_fn(ctxt: &mut Ctxt, f: impl FnOnce(&mut Ctxt)) -> FnId {
    let n = ctxt.ir.fns.len();
    let mut blocks: HashMap<_, _> = Default::default();
    blocks.insert(0, Vec::new());

    ctxt.ir.fns.insert(
        n,
        Function {
            blocks,
            start_block: 0,
        },
    );

    ctxt.stack.push(FnCtxt {
        node_ctr: 0,
        current_fn: n,
        current_blk: 0,
        loop_stack: Vec::new(),
        namespace_node: 0,
        global_node: 0,
        singletons_node: 0,
        ast_ptr: 0 as _,
    });

    f(ctxt);

    ctxt.stack.pop();

    n
}
