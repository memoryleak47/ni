use crate::*;

fn lower_expr(expr: &ASTExpr, block: &mut Block, id_ctr: &mut usize) -> Node {
	match expr {
		ASTExpr::Int(i) => {
			let n = *id_ctr; *id_ctr += 1;
			block.push(Statement::Compute(n, Expr::Num(r64(*i as f64))));
			n
		},
		_ => todo!("{:?}", expr)
	}
}

pub fn lower(ast: &AST) -> IR {
	let mut id_ctr = 0;
	let mut block = Vec::new();

	for stmt in ast {
		match stmt {
			ASTStatement::Expr(ASTExpr::FnCall(f, args)) => {
				if let ASTExpr::Var(fn_name) = &**f && fn_name == "print" {
					let n = lower_expr(&args[0], &mut block, &mut id_ctr);
					block.push(Statement::Print(n));
				}
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
