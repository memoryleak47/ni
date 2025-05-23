use crate::*;

mod init;
pub use init::*;

mod ctxt;
pub use ctxt::*;

mod fl;
pub use fl::*;

mod expr;
pub use expr::*;

mod stmt;
pub use stmt::*;


pub fn lower(ast: &AST) -> IR {
    let nameres_tab = nameres(ast);

    let mut ctxt = Ctxt {
        stack: Vec::new(),
        nameres_tab,
        ir: IR { main_fn: 0, fns: Default::default() },
        builtin_fns: Default::default(),
    };

    let main = new_fn(&mut ctxt, |ctxt| {
        let t = ctxt.push_table();
        ctxt.f_mut().lowering = Some(FnLowerCtxt {
            singletons_node: 0, // will be set in "add_builtins_and_singletons".
            // for the main function, the global scope is actually it's local scope.
            global_node: t,
            namespace_node: t,
            arg_node: ctxt.push_arg(),
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
