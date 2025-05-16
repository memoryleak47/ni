use crate::*;

fn lower_expr(expr: &ASTExpr, block: &mut Block, ctxt: &mut Ctxt) -> Node {
	match expr {
		ASTExpr::Int(i) => {
			let n = ctxt.node_ctr; ctxt.node_ctr += 1;
			block.push(Statement::Compute(n, Expr::Num(r64(*i as f64))));
			n
		},
		ASTExpr::Str(s) => {
			let n = ctxt.node_ctr; ctxt.node_ctr += 1;
			block.push(Statement::Compute(n, Expr::Str(s.to_string())));
			n
		},
		ASTExpr::BinOp(op, lhs, rhs) => {
			let lhs = lower_expr(lhs, block, ctxt);
			let rhs = lower_expr(rhs, block, ctxt);
			let n = ctxt.node_ctr; ctxt.node_ctr += 1;
			block.push(Statement::Compute(n, Expr::BinOp(*op, lhs, rhs)));
			n
		},
		ASTExpr::Var(v) => {
			let v = ctxt.varmap[v];
			let n = ctxt.node_ctr; ctxt.node_ctr += 1;
			let idx = lower_expr(&ASTExpr::Int(0), block, ctxt);
			block.push(Statement::Compute(n, Expr::Index(v, idx)));
			n
		},
		_ => todo!("{:?}", expr)
	}
}

struct Ctxt {
	node_ctr: usize,
	varmap: Map<String, Node>,
}

pub fn lower(ast: &AST) -> IR {
	let mut block = Vec::new();
	let mut ctxt = Ctxt {
		node_ctr: 0,
		varmap: Map::new(),
	};

	for stmt in ast {
		match stmt {
			ASTStatement::Expr(ASTExpr::FnCall(f, args)) => {
				if let ASTExpr::Var(fn_name) = &**f && fn_name == "print" {
					let n = lower_expr(&args[0], &mut block, &mut ctxt);
					block.push(Statement::Print(n));
				}
			},
			ASTStatement::Assign(ASTExpr::Var(v), rhs) => {
				if !ctxt.varmap.contains_key(&**v) {
					let n = ctxt.node_ctr; ctxt.node_ctr += 1;
					block.push(Statement::Compute(n, Expr::NewTable));
					ctxt.varmap.insert(v.clone(), n);
				}
				let var = ctxt.varmap[&**v];
				let idx = lower_expr(&ASTExpr::Int(0), &mut block, &mut ctxt);
				let val = lower_expr(rhs, &mut block, &mut ctxt);
				block.push(Statement::Store(var, idx, val));
			},
			_ => todo!(),
		}
	}

	block.push(Statement::Return);

	let mut blocks: HashMap<BlockId, Block> = Default::default();
	blocks.insert(0, block);

	let mut fns: HashMap<FnId, Function> = Default::default();
	fns.insert(0, Function {
		start_block: 0,
		blocks,
	});
	IR {
		main_fn: 0,
		fns,
	}
}
