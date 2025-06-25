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
    let userstart = Symbol::new("userstart".to_string());
    let mut ctxt = Ctxt {
        stack: vec![FnCtxt {
            current_pid: userstart,
            lowering: Some(FnLowerCtxt {
                ast_ptr: 0 as *const _,
            }),
        }],
        nameres_tab,
        procs: Map::new(),
    };

    ctxt.procs.insert(userstart, Vec::new());

    for stmt in &**ast {
        lower_stmt(stmt, &mut ctxt);
    }

    ctxt.push(format!("exit"));

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
        ASTStatement::Assign(ASTExpr::Var(v), rhs) => {
            let ns = find_namespace(v, ctxt);
            let rhs = lower_expr(rhs, ctxt);
            ctxt.push(format!("{ns}[\"{v}\"] = {rhs}"));
        },
        _ => todo!(),
    }
}

fn lower_expr(e: &ASTExpr, ctxt: &mut Ctxt) -> String {
    match e {
        ASTExpr::FnCall(f, args) => {
            let f = lower_expr(f, ctxt);
            let suc = ctxt.f().current_pid.next_fresh();
            ctxt.push(format!("%new_f = {{}}"));
            ctxt.push(format!("%new_f.retpid = {suc}"));
            ctxt.push(format!("%new_f.retval = {{}}"));
            ctxt.push(format!("%new_f.arg = {{}}"));
            for (i, a) in args.iter().enumerate() {
                let a = lower_expr(a, ctxt);
                ctxt.push(format!("%new_f.arg[{i}] = {a}"));
            }
            ctxt.push(format!("@.frame = %new_f"));
            ctxt.push(format!("jmp {f}.pid"));

            ctxt.procs.insert(suc, vec![]);
            ctxt.stack.last_mut().unwrap().current_pid = suc;

            String::new() // TODO
        },
        ASTExpr::Var(v) => {
            let ns = find_namespace(v, ctxt);
            format!("{ns}[\"{v}\"]")
        },
        ASTExpr::Str(s) => format!("\"{s}\""),
        ASTExpr::Int(i) => format!("{i}"),
        ASTExpr::Bool(true) => format!("True"),
        ASTExpr::Bool(false) => format!("False"),
        _ => todo!("{:?}", e),
    }
}

pub fn find_namespace(v: &str, ctxt: &mut Ctxt) -> String {
    let k = (ctxt.fl().ast_ptr, v.to_string());
    match ctxt.nameres_tab.get(&k) {
        Some(VarPlace::Local) => format!("@.frame.pylocals"),
        _ => format!("@.globals"),
    }
}
