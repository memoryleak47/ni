use crate::*;

fn lower_expr(expr: &ASTExpr, block: &mut Block) -> Node {
	todo!("{:?}", expr)
}

pub fn lower(ast: &AST) -> IR {
	let mut block = Vec::new();

	for stmt in ast {
		match stmt {
			ASTStatement::Expr(ASTExpr::FnCall(f, args)) => {
				if let ASTExpr::Var(fn_name) = &**f && fn_name == "print" {
					let n = lower_expr(&args[0], &mut block);
					block.push(Statement::Print(n));
				}
			},
			_ => todo!(),
		}
	}

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
