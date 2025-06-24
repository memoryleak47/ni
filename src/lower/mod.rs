use crate::*;

/*
mod init;
pub use init::*;

mod nameres;
pub use nameres::*;

mod ctxt;
pub use ctxt::*;

mod fl;
pub use fl::*;

mod expr;
pub use expr::*;

mod stmt;
pub use stmt::*;
*/

pub fn lower(ast: &AST) -> IR {
    let mut entries = std::fs::read_dir("./src/sem").unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>().unwrap();
    entries.sort();

    let mut ir = IR {
        procs: Map::new(),
        main_pid: ProcId(gsymb_add("todo".to_string())),
    };
    for x in entries {
        let s = std::fs::read_to_string(x).unwrap();
        let toks = ir_tokenize(&s);
        let ir2 = ir_assemble(&toks[..]);
        ir.procs.extend(ir2.procs);
    }

    ir
}

/*
pub fn lower(ast: &AST) -> IR {
    let nameres_tab = nameres(ast);

    let mut ctxt = Ctxt {
        stack: Vec::new(),
        nameres_tab,
        ir: IR { main_fn: 0, fns: Default::default() },
        builtin_fns: Default::default(),
    };

    let main = new_fn_general(true, &mut ctxt, |ctxt| {
        let t = ctxt.push_table();
        ctxt.f_mut().lowering = Some(FnLowerCtxt {
            // for the main function, the global scope is actually it's local scope.
            global_node: t,
            namespace_node: t,
            ast_ptr: 0 as _,
            loop_stack: Vec::new(),
        });


        add_builtins_and_singletons(ctxt);
        lower_ast(ast, ctxt);

        ctxt.push_return();
    });

    ctxt.ir.main_fn = main;
    ctxt.ir
}

pub fn find_namespace(v: &str, ctxt: &mut Ctxt) -> Node {
    let k = (ctxt.fl().ast_ptr, v.to_string());
    match ctxt.nameres_tab.get(&k) {
        Some(VarPlace::Local) => ctxt.fl().namespace_node,
        _ => ctxt.fl().global_node,
    }
}
*/
