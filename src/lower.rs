use crate::*;

fn lower_expr(expr: &ASTExpr, ctxt: &mut Ctxt) -> Node {
	match expr {
		ASTExpr::Int(i) => {
			ctxt.push_compute(Expr::Num(r64(*i as f64)))
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
}

pub fn lower(ast: &AST) -> IR {
	let mut ctxt = Ctxt::new();

	for stmt in ast {
		match stmt {
			ASTStatement::Expr(ASTExpr::FnCall(f, args)) => {
				if let ASTExpr::Var(fn_name) = &**f && fn_name == "print" {
					let n = lower_expr(&args[0], &mut ctxt);
					ctxt.push_statement(Statement::Print(n));
				}
			},
			ASTStatement::Assign(ASTExpr::Var(v), rhs) => {
				if !ctxt.varmap.contains_key(&**v) {
					let n = ctxt.push_compute(Expr::NewTable);
					ctxt.varmap.insert(v.clone(), n);
				}
				let var = ctxt.varmap[&**v];
				let idx = lower_expr(&ASTExpr::Int(0), &mut ctxt);
				let val = lower_expr(rhs, &mut ctxt);
				ctxt.push_statement(Statement::Store(var, idx, val));
			},
			_ => todo!(),
		}
	}

	ctxt.push_statement(Statement::Return);
	ctxt.ir
}
