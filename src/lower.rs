use crate::*;

fn add_builtins(ctxt: &mut Ctxt) {
	let n = ctxt.ir.fns.len();
	let mut blocks: HashMap<_, _> = Default::default();
	blocks.insert(0, vec![
		Statement::Compute(0, Expr::Arg),
		Statement::Compute(1, Expr::Int(0)),
		Statement::Compute(2, Expr::Index(0, 1)),
		Statement::Print(2),

		Statement::Compute(3, Expr::Str("ret".to_string())),
		Statement::Compute(4, Expr::None),
		Statement::Store(0, 3, 4),
		Statement::Return,
	]);
	ctxt.ir.fns.insert(n, Function {
		blocks,
		start_block: 0,
	});
	let print_inner = ctxt.push_compute(Expr::Function(n));
	let print = ctxt.push_compute(Expr::NewTable);
	let idx = ctxt.push_compute(Expr::Int(0));
	ctxt.push_statement(Statement::Store(print, idx, print_inner));
	ctxt.f_mut().varmap.insert(String::from("print"), print);
}

fn lower_expr(expr: &ASTExpr, ctxt: &mut Ctxt) -> Node {
	match expr {
		ASTExpr::None => {
			ctxt.push_compute(Expr::None)
		},
		ASTExpr::Int(i) => {
			ctxt.push_compute(Expr::Int(*i))
		},
		ASTExpr::Bool(b) => {
			ctxt.push_compute(Expr::Bool(*b))
		},
		ASTExpr::Str(s) => {
			ctxt.push_compute(Expr::Str(s.to_string()))
		},
		ASTExpr::BinOp(op, lhs, rhs) => {
			let lhs = lower_expr(lhs, ctxt);
			let rhs = lower_expr(rhs, ctxt);
			ctxt.push_compute(Expr::BinOp(*op, lhs, rhs))
		},
		ASTExpr::Var(v) => {
			let v = ctxt.f().varmap[v];
			let idx = lower_expr(&ASTExpr::Int(0), ctxt);
			ctxt.push_compute(Expr::Index(v, idx))
		},
		ASTExpr::FnCall(f, args) => {
			let f = lower_expr(&f, ctxt);
			let arg = ctxt.push_compute(Expr::NewTable);
			for (i, a) in args.iter().enumerate() {
				let i = ctxt.push_compute(Expr::Int(i as _));
				let v = lower_expr(a, ctxt);
				ctxt.push_statement(Statement::Store(arg, i, v));
			}
			ctxt.push_statement(Statement::FnCall(f, arg));
			let idx = ctxt.push_compute(Expr::Str("ret".to_string()));
			ctxt.push_compute(Expr::Index(arg, idx))
		},
		_ => todo!("{:?}", expr)
	}
}

struct FnCtxt {
	node_ctr: usize,
	varmap: Map<String, Node>,
	current_fn: FnId,
	current_blk: BlockId,
	loop_stack: Vec<(/*break*/BlockId, /*continue*/BlockId)>,
}

impl FnCtxt {
	fn new(f: FnId) -> Self {
		Self {
			node_ctr: 0,
			varmap: Default::default(),
			current_fn: f,
			current_blk: 0,
			loop_stack: Vec::new(),
		}
	}
}

struct Ctxt {
	stack: Vec<FnCtxt>,
	ir: IR,
}

impl Ctxt {
	fn f(&self) -> &FnCtxt { self.stack.last().unwrap() }
	fn f_mut(&mut self) -> &mut FnCtxt { self.stack.last_mut().unwrap() }

	fn new() -> Self {
		let mut fns: HashMap<_, _> = Default::default();
		let mut blocks: HashMap<_, _> = Default::default();
		blocks.insert(0, Vec::new());
		let main_fn = Function {
			blocks,
			start_block: 0,
		};
		fns.insert(0, main_fn);

		Ctxt {
			stack: vec![FnCtxt::new(0)],
			ir: IR {
				main_fn: 0,
				fns,
			},
		}
	}

	fn push_compute(&mut self, expr: Expr) -> Node {
		let n = self.f().node_ctr; self.f_mut().node_ctr += 1;
		self.push_statement(Statement::Compute(n, expr));
		n
	}

	fn push_statement(&mut self, stmt: Statement) {
		let current_fn = self.f().current_fn;
		let current_blk = self.f().current_blk;
		self.ir.fns.get_mut(&current_fn).unwrap().blocks.get_mut(&current_blk).unwrap().push(stmt);
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
	if !ctxt.f().varmap.contains_key(v) {
		let n = ctxt.push_compute(Expr::NewTable);
		ctxt.f_mut().varmap.insert(v.to_string(), n);
	}
	let var = ctxt.f().varmap[v];
	let idx = lower_expr(&ASTExpr::Int(0), ctxt);
	ctxt.push_statement(Statement::Store(var, idx, val));
}

fn lower_ast(ast: &AST, ctxt: &mut Ctxt) {
	for stmt in ast {
		match stmt {
			ASTStatement::Expr(e) => {
				lower_expr(e, ctxt);
			},
			ASTStatement::Assign(ASTExpr::Var(v), rhs) => {
				let val = lower_expr(rhs, ctxt);
				lower_assign(v, val, ctxt);
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
			},
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
			},
			ASTStatement::Def(name, args, body) => {
				let i = ctxt.ir.fns.len();
				ctxt.stack.push(FnCtxt::new(i));

				{ // add empty fn to IR
					let mut blocks: HashMap<_, _> = Default::default();
					blocks.insert(0, Vec::new());
					let f = Function { blocks, start_block: 0 };
					ctxt.ir.fns.insert(i, f);
				}

				// load args
				let argtable = ctxt.push_compute(Expr::Arg);
				let zero = ctxt.push_compute(Expr::Int(0));
				for (i, a) in args.iter().enumerate() {
					let i = ctxt.push_compute(Expr::Int(i as _));
					let val = ctxt.push_compute(Expr::Index(argtable, i));
					let t = ctxt.push_compute(Expr::NewTable);
					ctxt.push_statement(Statement::Store(t, zero, val));
					ctxt.f_mut().varmap.insert(a.to_string(), t);
				}

				lower_ast(body, ctxt);
				ctxt.push_statement(Statement::Return);

				ctxt.stack.pop();

				let val = ctxt.push_compute(Expr::Function(i));
				lower_assign(name, val, ctxt);
			},
			ASTStatement::Return(opt) => {
				let expr = opt.as_ref().unwrap_or(&ASTExpr::None);
				let val = lower_expr(expr, ctxt);
				let argtable = ctxt.push_compute(Expr::Arg);
				let idx = ctxt.push_compute(Expr::Str("ret".to_string()));
				ctxt.push_statement(Statement::Store(argtable, idx, val));
				ctxt.push_statement(Statement::Return);
			},
			ASTStatement::Pass => {}, // do nothing
			ASTStatement::Break => {
				ctxt.push_goto(ctxt.f().loop_stack.last().unwrap().0);
			},
			ASTStatement::Continue => {
				ctxt.push_goto(ctxt.f().loop_stack.last().unwrap().1);
			},
			x => todo!("{:?}", x),
		}
	}

}

pub fn lower(ast: &AST) -> IR {
	let mut ctxt = Ctxt::new();

	add_builtins(&mut ctxt);
	lower_ast(ast, &mut ctxt);

	ctxt.push_statement(Statement::Return);
	ctxt.ir
}
