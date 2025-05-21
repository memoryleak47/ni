use crate::*;

fn new_fn(ctxt: &mut Ctxt, f: impl FnOnce(&mut Ctxt)) -> FnId {
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

fn add_print_builtin(ctxt: &mut Ctxt) {
    let print_fn = new_fn(ctxt, |ctxt| {
        let arg = ctxt.push_compute(Expr::Arg);
        let zero = ctxt.push_compute(Expr::Int(0));
        let first_arg = ctxt.push_compute(Expr::Index(arg, zero));
        ctxt.push_statement(Statement::Print(first_arg));
        let none = ctxt.push_compute(Expr::None);
        ctxt.push_store_str(arg, "ret", none);
        ctxt.push_statement(Statement::Return);
    });
    let print_f = ctxt.push_compute(Expr::Function(print_fn));
    let function = ctxt.push_compute_index_str(ctxt.f().singletons_node, "function");
    let print = build_value(print_f, function, ctxt);
    let nn = ctxt.f().namespace_node;
    ctxt.push_store_str(nn, "print", print);
}

fn build_value(payload: Node, type_: Node, ctxt: &mut Ctxt) -> Node {
    let t = ctxt.push_compute(Expr::NewTable);
    ctxt.push_store_str(t, "type", type_);
    let dict = ctxt.push_compute(Expr::NewTable);
    ctxt.push_store_str(t, "payload", payload);
    ctxt.push_store_str(t, "dict", dict);
    t
}

fn add_construct_builtin(ctxt: &mut Ctxt) {
    new_fn(ctxt, |ctxt| {
        let arg = ctxt.push_compute(Expr::Arg);
        let t = ctxt.push_compute(Expr::NewTable);
        ctxt.push_store_str(arg, "ret", t);
        ctxt.push_statement(Statement::Return);
    });
}

fn add_singletons(ctxt: &mut Ctxt) {
    let singleton = ctxt.push_compute(Expr::NewTable);
    ctxt.f_mut().singletons_node = singleton;

    let type_ = ctxt.push_compute(Expr::NewTable);
    let type_str = ctxt.push_str("type");
    ctxt.push_store(singleton, type_str, type_);

    // type is of type `type`.
    ctxt.push_store(type_, type_str, type_);

    let mut add_primitive_type = |name| {
        let tab = ctxt.push_compute(Expr::NewTable);
        let name_str = ctxt.push_str(name);
        ctxt.push_store(tab, type_str, type_);
        ctxt.push_store(singleton, name_str, tab);
    };

    add_primitive_type("function");
    add_primitive_type("str");
    add_primitive_type("int");
    add_primitive_type("float");
    add_primitive_type("bool");
    add_primitive_type("NoneType");
}

fn add_builtins_and_singletons(ctxt: &mut Ctxt) {
    add_singletons(ctxt);
    add_print_builtin(ctxt);
    add_construct_builtin(ctxt);
}

fn lower_expr(expr: &ASTExpr, ctxt: &mut Ctxt) -> Node {
    match expr {
        ASTExpr::None => ctxt.push_compute(Expr::None),
        ASTExpr::Int(i) => ctxt.push_compute(Expr::Int(*i)),
        ASTExpr::Bool(b) => ctxt.push_compute(Expr::Bool(*b)),
        ASTExpr::Str(s) => ctxt.push_compute(Expr::Str(s.to_string())),
        ASTExpr::BinOp(op, lhs, rhs) => {
            let lhs = lower_expr(lhs, ctxt);
            let rhs = lower_expr(rhs, ctxt);
            ctxt.push_compute(Expr::BinOp(*op, lhs, rhs))
        }
        ASTExpr::Var(v) => {
            let v_str = lower_expr(&ASTExpr::Str(v.to_string()), ctxt);
            let nn = if let Some(VarPlace::Local) =
                ctxt.nameres_tab.get(&(ctxt.f().ast_ptr, v.to_string()))
            {
                ctxt.f().namespace_node
            } else {
                ctxt.f().global_node
            };
            ctxt.push_compute(Expr::Index(nn, v_str))
        }
        ASTExpr::FnCall(f, args) => lower_fn_call(&*f, args, ctxt),
        ASTExpr::Attribute(e, a) => {
            let e = lower_expr(e, ctxt);
            let a = ctxt.push_compute(Expr::Str(a.to_string()));
            ctxt.push_compute(Expr::Index(e, a))
        },
        _ => todo!("{:?}", expr),
    }
}

fn lower_fn_call(f: &ASTExpr, args: &[ASTExpr], ctxt: &mut Ctxt) -> Node {
    let f = lower_expr(&f, ctxt);
    let arg = ctxt.push_compute(Expr::NewTable);

    let is_function_ty = ctxt.alloc_blk();
    let is_no_function_ty = ctxt.alloc_blk();
    let is_class = ctxt.alloc_blk();
    let err = ctxt.alloc_blk();
    let go = ctxt.alloc_blk();

    // where we store the function to call (under index "0").
    let tmp = ctxt.push_compute(Expr::NewTable);

    // if f["type"] == singletons["function"]: goto is_function_ty | is_no_function_ty
    let a = ctxt.push_compute_index_str(f, "type");
    let b = ctxt.push_compute_index_str(ctxt.f().singletons_node, "function");
    let cond = ctxt.push_compute(Expr::BinOp(BinOpKind::IsEqual, a, b));
    ctxt.push_if(cond, is_function_ty, is_no_function_ty);

    ctxt.focus_blk(is_function_ty);
    let f_payload = ctxt.push_compute_index_str(f, "payload");
    ctxt.push_store_str(tmp, "0", f_payload);
    ctxt.push_goto(go);

    // if f["type"] == singletons["type"]: goto is_class | err
    ctxt.focus_blk(is_no_function_ty);
    let a = ctxt.push_compute_index_str(f, "type");
    let b = ctxt.push_compute_index_str(ctxt.f().singletons_node, "type");
    let cond = ctxt.push_compute(Expr::BinOp(BinOpKind::IsEqual, a, b));
    ctxt.push_if(cond, is_class, err);

    ctxt.focus_blk(is_class);
    let payload = ctxt.push_compute(Expr::Function(2)); // construct!
    ctxt.push_store_str(tmp, "0", payload);
    ctxt.push_goto(go);

    ctxt.focus_blk(err);
    ctxt.push_statement(Statement::Throw("can't call this thing!".to_string()));

    ctxt.focus_blk(go);

    // pass "scope_global" along.
    let scope_global_str = ctxt.push_str("scope_global");
    ctxt.push_statement(Statement::Store(
        arg,
        scope_global_str,
        ctxt.f().global_node,
    ));

    // pass "singletons" along.
    let singletons_str = ctxt.push_str("singletons");
    ctxt.push_statement(Statement::Store(
        arg,
        singletons_str,
        ctxt.f().singletons_node,
    ));

    for (i, a) in args.iter().enumerate() {
        let i = ctxt.push_compute(Expr::Int(i as _));
        let v = lower_expr(a, ctxt);
        ctxt.push_statement(Statement::Store(arg, i, v));
    }
    let f_payload = ctxt.push_compute_index_str(tmp, "0");
    ctxt.push_statement(Statement::FnCall(f_payload, arg));
    let idx = ctxt.push_compute(Expr::Str("ret".to_string()));
    ctxt.push_compute(Expr::Index(arg, idx))
}

struct FnCtxt {
    node_ctr: usize,
    current_fn: FnId,
    current_blk: BlockId,
    loop_stack: Vec<(/*break*/ BlockId, /*continue*/ BlockId)>,
    namespace_node: Node,
    global_node: Node,
    singletons_node: Node,
    ast_ptr: *const ASTStatement, // the original def stmt
}

impl FnCtxt {
    fn new(f: FnId, ast_ptr: *const ASTStatement) -> Self {
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

struct Ctxt {
    stack: Vec<FnCtxt>,
    nameres_tab: NameResTable,
    ir: IR,
}

impl Ctxt {
    fn f(&self) -> &FnCtxt {
        self.stack.last().unwrap()
    }
    fn f_mut(&mut self) -> &mut FnCtxt {
        self.stack.last_mut().unwrap()
    }

    fn new(ast: &AST) -> Self {
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
        }
    }

    fn push_compute(&mut self, expr: Expr) -> Node {
        let n = self.f().node_ctr;
        self.f_mut().node_ctr += 1;
        self.push_statement(Statement::Compute(n, expr));
        n
    }

    fn push_str(&mut self, s: &str) -> Node {
        self.push_compute(Expr::Str(s.to_string()))
    }

    fn push_store(&mut self, t: Node, k: Node, v: Node) {
        self.push_statement(Statement::Store(t, k, v));
    }

    fn push_store_str(&mut self, t: Node, k: &str, v: Node) {
        let k = self.push_str(k);
        self.push_statement(Statement::Store(t, k, v));
    }

    fn push_compute_index_str(&mut self, t: Node, k: &str) -> Node {
        let k = self.push_str(k);
        self.push_compute(Expr::Index(t, k))
    }

    fn push_statement(&mut self, stmt: Statement) {
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

    fn push_goto(&mut self, b: BlockId) {
        let true_ = lower_expr(&ASTExpr::Bool(true), self);
        self.push_statement(Statement::If(true_, b, b));
    }

    fn push_if(&mut self, cond: Node, b1: BlockId, b2: BlockId) {
        self.push_statement(Statement::If(cond, b1, b2));
    }

    fn alloc_blk(&mut self) -> BlockId {
        let current_fn = self.f().current_fn;
        let f = self.ir.fns.get_mut(&current_fn).unwrap();
        let n = f.blocks.len();
        f.blocks.insert(n, Vec::new());
        n
    }

    fn focus_blk(&mut self, b: BlockId) {
        self.f_mut().current_blk = b;
    }
}

fn lower_assign(v: &str, val: Node, ctxt: &mut Ctxt) {
    let v_str = lower_expr(&ASTExpr::Str(v.to_string()), ctxt);

    let nn = if let Some(VarPlace::Local) = ctxt.nameres_tab.get(&(ctxt.f().ast_ptr, v.to_string()))
    {
        ctxt.f().namespace_node
    } else {
        ctxt.f().global_node
    };
    ctxt.push_statement(Statement::Store(nn, v_str, val));
}

fn lower_ast(ast: &AST, ctxt: &mut Ctxt) {
    for stmt in ast {
        match stmt {
            ASTStatement::Expr(e) => {
                lower_expr(e, ctxt);
            }
            ASTStatement::Assign(ASTExpr::Var(v), rhs) => {
                let val = lower_expr(rhs, ctxt);
                lower_assign(v, val, ctxt);
            }
            ASTStatement::Assign(ASTExpr::Attribute(e, v), rhs) => {
                let e = lower_expr(e, ctxt);
                let val = lower_expr(rhs, ctxt);
                let v_str = ctxt.push_compute(Expr::Str(v.to_string()));
                ctxt.push_statement(Statement::Store(e, v_str, val));
            },
            ASTStatement::If(cond, then) => {
                let cond = lower_expr(cond, ctxt);
                let b = ctxt.alloc_blk();
                let post = ctxt.alloc_blk();
                ctxt.push_statement(Statement::If(cond, b, post));

                ctxt.focus_blk(b);
                lower_ast(then, ctxt);
                ctxt.push_goto(post);

                ctxt.focus_blk(post);
            }
            ASTStatement::While(cond, then) => {
                let pre = ctxt.alloc_blk();
                let b = ctxt.alloc_blk();
                let post = ctxt.alloc_blk();
                ctxt.f_mut().loop_stack.push((post, pre));

                ctxt.push_goto(pre);
                ctxt.focus_blk(pre);
                let cond = lower_expr(cond, ctxt);

                ctxt.push_statement(Statement::If(cond, b, post));

                ctxt.focus_blk(b);
                lower_ast(then, ctxt);
                ctxt.push_goto(pre);

                ctxt.focus_blk(post);
                ctxt.f_mut().loop_stack.pop();
            }
            ASTStatement::Def(name, args, body) => {
                let i = ctxt.ir.fns.len();
                ctxt.stack.push(FnCtxt::new(i, stmt as _));

                {
                    // add empty fn to IR
                    let mut blocks: HashMap<_, _> = Default::default();

                    // TODO unify this with the construction of the main function.
                    // this creates the namespace node.
                    blocks.insert(
                        0,
                        vec![
                            Statement::Compute(0, Expr::NewTable),
                            Statement::Compute(1, Expr::Arg),
                            Statement::Compute(2, Expr::Str("scope_global".to_string())),
                            Statement::Compute(3, Expr::Index(1, 2)),
                            Statement::Compute(4, Expr::Str("singletons".to_string())),
                            Statement::Compute(5, Expr::Index(1, 4)),
                        ],
                    );
                    let f = Function {
                        blocks,
                        start_block: 0,
                    };
                    ctxt.ir.fns.insert(i, f);
                }

                // load args
                let argtable = ctxt.push_compute(Expr::Arg);
                for (i, a) in args.iter().enumerate() {
                    let i = ctxt.push_compute(Expr::Int(i as _));
                    let val = ctxt.push_compute(Expr::Index(argtable, i));
                    let nn = ctxt.f().namespace_node;
                    let a_str = ctxt.push_compute(Expr::Str(a.to_string()));
                    ctxt.push_statement(Statement::Store(nn, a_str, val));
                }

                lower_ast(body, ctxt);
                ctxt.push_statement(Statement::Return);

                ctxt.stack.pop();

                let function = ctxt.push_compute(Expr::Function(i));
                let function_t = ctxt.push_compute_index_str(ctxt.f().singletons_node, "function");
                let val = build_value(function, function_t, ctxt);

                lower_assign(name, val, ctxt);
            }
            ASTStatement::Return(opt) => {
                let expr = opt.as_ref().unwrap_or(&ASTExpr::None);
                let val = lower_expr(expr, ctxt);
                let argtable = ctxt.push_compute(Expr::Arg);
                let idx = ctxt.push_compute(Expr::Str("ret".to_string()));
                ctxt.push_statement(Statement::Store(argtable, idx, val));
                ctxt.push_statement(Statement::Return);
            }
            ASTStatement::Pass => {} // do nothing
            ASTStatement::Break => {
                ctxt.push_goto(ctxt.f().loop_stack.last().unwrap().0);
            }
            ASTStatement::Continue => {
                ctxt.push_goto(ctxt.f().loop_stack.last().unwrap().1);
            }
            ASTStatement::Scope(..) => {} // scope is already handled in nameres
            ASTStatement::Class(name, _args, body) => {
                // TODO: most stuff is missing here.

                lower_ast(body, ctxt);
                let val = ctxt.push_compute(Expr::NewTable); // the construct builtin
                let type_ = ctxt.push_compute_index_str(ctxt.f().singletons_node, "type");
                ctxt.push_store_str(val, "type", type_);
                lower_assign(name, val, ctxt);
            }
            x => todo!("{:?}", x),
        }
    }
}

pub fn lower(ast: &AST) -> IR {
    let mut ctxt = Ctxt::new(ast);

    add_builtins_and_singletons(&mut ctxt);
    lower_ast(ast, &mut ctxt);

    ctxt.push_statement(Statement::Return);
    ctxt.ir
}
