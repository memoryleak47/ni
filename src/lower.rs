use crate::*;

fn lower_expr(expr: &ASTExpr, ctxt: &mut Ctxt) -> Node {
	match expr {
		ASTExpr::Int(i) => {
			ctxt.push_compute(Expr::Num(r64(*i as f64)))
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
			let n = ctxt.node_ctr; ctxt.node_ctr += 1;
			ctxt.push_compute(Expr::BinOp(*op, lhs, rhs))
		},
		ASTExpr::Var(v) => {
			let v = ctxt.varmap[v];
			let idx = lower_expr(&ASTExpr::Int(0), ctxt);
			ctxt.push_compute(Expr::Index(v, idx))
		},
		_ => todo!("{:?}", expr)
	}
}

struct Ctxt {
	node_ctr: usize,
	varmap: Map<String, Node>,
	current_fn: FnId,
	current_blk: BlockId,
	ir: IR,
}

impl Ctxt {
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
			node_ctr: 0,
			varmap: Map::new(),
			current_fn: 0,
			current_blk: 0,
			ir: IR {
				main_fn: 0,
				fns,
			},
		}
	}

	fn push_compute(&mut self, expr: Expr) -> Node {
		let n = self.node_ctr; self.node_ctr += 1;
		self.push_statement(Statement::Compute(n, expr));
		n
	}

	fn push_statement(&mut self, stmt: Statement) {
		self.ir.fns.get_mut(&self.current_fn).unwrap().blocks.get_mut(&self.current_blk).unwrap().push(stmt);
	}

	fn push_goto(&mut self, b: BlockId) {
		let true_ = lower_expr(&ASTExpr::Bool(true), self);
		self.push_statement(Statement::If(true_, b, b));
	}

	fn alloc_blk(&mut self) -> BlockId {
		let f = self.ir.fns.get_mut(&self.current_fn).unwrap();
		let n = f.blocks.len();
		f.blocks.insert(n, Vec::new());
		n
	}

	fn focus_blk(&mut self, b: BlockId) {
		self.current_blk = b;
	}
}

fn lower_ast(ast: &AST, ctxt: &mut Ctxt) {
	for stmt in ast {
		match stmt {
			ASTStatement::Expr(ASTExpr::FnCall(f, args)) => {
				if let ASTExpr::Var(fn_name) = &**f && fn_name == "print" {
					let n = lower_expr(&args[0], ctxt);
					ctxt.push_statement(Statement::Print(n));
				}
			},
			ASTStatement::Assign(ASTExpr::Var(v), rhs) => {
				if !ctxt.varmap.contains_key(&**v) {
					let n = ctxt.push_compute(Expr::NewTable);
					ctxt.varmap.insert(v.clone(), n);
				}
				let var = ctxt.varmap[&**v];
				let idx = lower_expr(&ASTExpr::Int(0), ctxt);
				let val = lower_expr(rhs, ctxt);
				ctxt.push_statement(Statement::Store(var, idx, val));
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
			_ => todo!(),
		}
	}

}

pub fn lower(ast: &AST) -> IR {
	let mut ctxt = Ctxt::new();

	lower_ast(ast, &mut ctxt);

	ctxt.push_statement(Statement::Return);
	ctxt.ir
}
