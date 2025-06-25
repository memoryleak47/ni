use crate::*;

mod nameres;
pub use nameres::*;

fn lower_ast(ast: &AST) -> String {
    let nameres_tab = nameres(ast);

    for stmt in &**ast {
        lower_stmt(stmt);
    }

    String::new()
}

fn lower_stmt(stmt: &ASTStatement) -> String {
    match stmt {
        ASTStatement::Expr(e) => lower_expr(e),
        _ => todo!(),
    }
}

fn lower_expr(e: &ASTExpr) -> String {
    match e {
        ASTExpr::Var(v) => todo!(),
        _ => todo!(),
    }
}

pub fn lower(ast: &AST) -> IR {
    let mut s = lower_ast(ast);
    s.extend(include_str!("../sem/init.ir").chars());

    let toks = ir_tokenize(&s);
    ir_assemble(&toks[..])
}
