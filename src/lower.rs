use crate::*;

fn add_print_builtin(ctxt: &mut Ctxt) {
    let n = ctxt.ir.fns.len();
    let mut blocks: HashMap<_, _> = Default::default();
    blocks.insert(
        0,
        vec![
            Statement::Compute(0, Expr::Arg),
            Statement::Compute(1, Expr::Int(0)),
            Statement::Compute(2, Expr::Index(0, 1)),
            Statement::Print(2),
            Statement::Compute(3, Expr::Str("ret".to_string())),
            Statement::Compute(4, Expr::None),
            Statement::Store(0, 3, 4),
            Statement::Return,
        ],
    );
    ctxt.ir.fns.insert(
        n,
        Function {
            blocks,
            start_block: 0,
        },
    );
    let print = ctxt.push_compute(Expr::Function(n));
    let print_str = ctxt.push_compute(Expr::Str("print".to_string()));
    let nn = ctxt.f().namespace_node;
    ctxt.push_statement(Statement::Store(nn, print_str, print));
}

fn add_construct_builtin(ctxt: &mut Ctxt) {
    let n = ctxt.ir.fns.len();
    let mut blocks: HashMap<_, _> = Default::default();
    blocks.insert(
        0,
        vec![
            Statement::Compute(0, Expr::Arg),
            Statement::Compute(1, Expr::Str("ret".to_string())),
            Statement::Compute(2, Expr::NewTable),
            Statement::Store(0, 1, 2),
            Statement::Return,
        ],
    );
    ctxt.ir.fns.insert(
        n,
        Function {
            blocks,
            start_block: 0,
        },
    );
}

fn add_builtins(ctxt: &mut Ctxt) {
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
        ASTExpr::FnCall(f, args) => {
            let f = lower_expr(&f, ctxt);
            let arg = ctxt.push_compute(Expr::NewTable);

            // pass "scope_global" along.
            let scope_global_str = ctxt.push_compute(Expr::Str("scope_global".to_string()));
            ctxt.push_statement(Statement::Store(
                arg,
                scope_global_str,
                ctxt.f().global_node,
            ));

            for (i, a) in args.iter().enumerate() {
                let i = ctxt.push_compute(Expr::Int(i as _));
                let v = lower_expr(a, ctxt);
                ctxt.push_statement(Statement::Store(arg, i, v));
            }
            ctxt.push_statement(Statement::FnCall(f, arg));
            let idx = ctxt.push_compute(Expr::Str("ret".to_string()));
            ctxt.push_compute(Expr::Index(arg, idx))
        }
        ASTExpr::Attribute(e, a) => {
            let e = lower_expr(e, ctxt);
            let a = ctxt.push_compute(Expr::Str(a.to_string()));
            ctxt.push_compute(Expr::Index(e, a))
        },
        _ => todo!("{:?}", expr),
    }
}

struct FnCtxt {
    node_ctr: usize,
    current_fn: FnId,
    current_blk: BlockId,
    loop_stack: Vec<(/*break*/ BlockId, /*continue*/ BlockId)>,
    namespace_node: Node,
    global_node: Node,
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
                            Statement::Compute(1, Expr::Str("scope_global".to_string())),
                            Statement::Compute(2, Expr::Arg),
                            Statement::Compute(3, Expr::Index(2, 1)),
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

                let val = ctxt.push_compute(Expr::Function(i));
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
                let val = ctxt.push_compute(Expr::Function(2)); // the construct builtin
                lower_assign(name, val, ctxt);
            }
            x => todo!("{:?}", x),
        }
    }
}

pub fn lower(ast: &AST) -> IR {
    let mut ctxt = Ctxt::new(ast);

    add_builtins(&mut ctxt);
    lower_ast(ast, &mut ctxt);

    ctxt.push_statement(Statement::Return);
    ctxt.ir
}
