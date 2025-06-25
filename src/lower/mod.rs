use crate::*;

mod nameres;
pub use nameres::*;

mod ctxt;
pub use ctxt::*;

fn lower_ast(ast: &AST) -> String {
    let nameres_tab = nameres(ast);

    let mut s = "proc userstart {\n".to_string();
    for stmt in &**ast {
        s.push_str(&*lower_stmt(stmt));
    }
    s.push_str("    exit;\n");
    s.push_str("}\n");
    s
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
