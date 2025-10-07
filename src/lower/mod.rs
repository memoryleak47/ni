use crate::*;

mod nameres;
pub use nameres::*;

mod ctxt;
use ctxt::*;

mod expr;
use expr::*;

mod body;
use body::*;

pub fn lower(ast: &AST) -> String {
    let mut s = String::from("#\n");
    s.extend(lower_ast(ast).chars());
    s.extend(include_str!(concat!(env!("OUT_DIR"), "/concat.ir")).chars());

    s
}

fn lower_ast(ast: &AST) -> String {
    let nameres_tab = nameres(ast);
    let userstart = Symbol::new("userstart".to_string());
    let mut ctxt = Ctxt {
        stack: vec![FnCtxt {
            current_pid: userstart,
            lowering: Some(FnLowerCtxt {
                loop_stack: Vec::new(),
                ast_ptr: 0 as *const _,
            }),
        }],
        nameres_tab,
        procs: Map::new(),
    };

    ctxt.procs.insert(userstart, Vec::new());

    lower_body(&**ast, &mut ctxt);

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
