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
    let mut ctxt = Ctxt {
        nameres_tab,
        procs: Map::new(),
    };

    ctxt.procs.insert(Symbol::new("userstart".to_string()), Vec::new());

    for stmt in &**ast {
        lower_stmt(stmt, &mut ctxt);
    }

    let mut s = String::new();
    for (pid, stmts) in ctxt.procs {
        s.push_str(&format!("proc {pid} {{\n"));
        for stmt in stmts {
            s.push_str(&format!("    {stmt};\n"));
        }
        s.push_str("}\n");
    }
    s
}

fn lower_stmt(stmt: &ASTStatement, ctxt: &mut Ctxt) {
    match stmt {
        ASTStatement::Expr(e) => {
            lower_expr(e, ctxt);
        },
        _ => todo!(),
    }
}

fn lower_expr(e: &ASTExpr, ctxt: &mut Ctxt) -> String {
    match e {
        ASTExpr::FnCall(f, args) => {
            let f = lower_expr(f, ctxt);
            let suc = Symbol::fresh();
            ctxt.push(format!("%new_f = {{}}"));
            ctxt.push(format!("%new_f.retpid = {suc}"));
            ctxt.push(format!("%new_f.retval = {{}}"));
            ctxt.push(format!("%new_f.arg = {{}}"));
            ctxt.push(format!("%new_f.arg[1] = 42"));
            ctxt.push(format!("@.frame = %new_f"));
            ctxt.push(format!("jmp {f}.pid"));

            ctxt.procs.insert(suc, vec!["exit".to_string()]);

            String::new() // TODO
        },
        ASTExpr::Var(v) => format!("@.globals[\"{v}\"]"),
        _ => todo!(),
    }
}
