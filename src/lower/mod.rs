use crate::*;

mod nameres;
pub use nameres::*;

mod ctxt;
pub use ctxt::*;

pub fn lower(ast: &AST) -> String {
    let mut s = lower_ast(ast);
    s.extend(include_str!("../sem/init.ir").chars());
    s
}

fn lower_ast(ast: &AST) -> String {
    let nameres_tab = nameres(ast);

    let mut s = "proc userstart {\n".to_string();
    for stmt in &**ast {
        s.push_str(&*lower_stmt(stmt));
    }
    // s.push_str("    exit;\n");
    s.push_str("}\n");
    s
}

fn lower_stmt(stmt: &ASTStatement) -> String {
    match stmt {
        ASTStatement::Expr(e) => format!("    {};\n", lower_expr(e)),
        _ => todo!(),
    }
}

fn lower_expr(e: &ASTExpr) -> String {
    match e {
        ASTExpr::FnCall(f, args) => format!("jmp {}.pid", lower_expr(f)),
        ASTExpr::Var(v) => format!("@.globals[\"{v}\"]"),
        _ => todo!(),
    }
}
