use crate::*;

mod init;
pub use init::*;

mod ctxt;
pub use ctxt::*;

mod fl;
pub use fl::*;

fn lower_expr(expr: &ASTExpr, ctxt: &mut Ctxt) -> Node {
    match expr {
        ASTExpr::None => ctxt.push_none(),
        ASTExpr::Int(i) => ctxt.push_int(*i),
        ASTExpr::Bool(b) => ctxt.push_bool(*b),
        ASTExpr::Str(s) => ctxt.push_str(s),
        ASTExpr::BinOp(op, lhs, rhs) => {
            let lhs = lower_expr(lhs, ctxt);
            let rhs = lower_expr(rhs, ctxt);
            ctxt.push_compute(Expr::BinOp(*op, lhs, rhs))
        }
        ASTExpr::Var(v) => {
            let nn = find_namespace(v, ctxt);
            ctxt.push_index_str(nn, v)
        }
        ASTExpr::FnCall(f, args) => lower_fn_call(&*f, args, ctxt),
        ASTExpr::Attribute(e, a) => {
            let found = ctxt.alloc_blk();
            let not_found = ctxt.alloc_blk();
            let inclass_found = ctxt.alloc_blk();
            let inclass_not_found = ctxt.alloc_blk();
            let post = ctxt.alloc_blk();

            let tmp = ctxt.push_table();
            let e = lower_expr(e, ctxt);
            let d = ctxt.push_index_str(e, "dict");
            let v = ctxt.push_index_str(d, a);
            ctxt.branch_undef(v, not_found, found);

            ctxt.focus_blk(found);
                ctxt.push_store_str(tmp, "0", v);
                ctxt.push_goto(post);

            ctxt.focus_blk(not_found);
                let class_t = ctxt.push_index_str(e, "type");
                let class_d = ctxt.push_index_str(class_t, "dict");
                let class_v = ctxt.push_index_str(class_d, a);
                ctxt.branch_undef(class_v, inclass_not_found, inclass_found);

            ctxt.focus_blk(inclass_found);
                ctxt.push_store_str(tmp, "0", class_v);
                ctxt.push_goto(post);

            ctxt.focus_blk(inclass_not_found);
                ctxt.push_statement(Statement::Throw("missing attribute!".to_string()));

            ctxt.focus_blk(post);
                ctxt.push_index_str(tmp, "0")
        },
        _ => todo!("{:?}", expr),
    }
}

// lower a function call, where you know that f is of type function.
// arg is empty thus far.
fn lower_fn_type_call(f: Node, args: &[Node], arg: Node, ctxt: &mut Ctxt) {
    ctxt.push_store_str(arg, "scope_global", ctxt.fl().global_node);
    ctxt.push_store_str(arg, "singletons", ctxt.fl().singletons_node);

    for (i, a) in args.iter().enumerate() {
        let i = ctxt.push_int(i as _);
        ctxt.push_store(arg, i, *a);
    }

    let f_payload = ctxt.push_index_str(f, "payload");
    ctxt.push_statement(Statement::FnCall(f_payload, arg));
}

fn lower_fn_call(f: &ASTExpr, args: &[ASTExpr], ctxt: &mut Ctxt) -> Node {
    let f = lower_expr(&f, ctxt);
    let mut args: Vec<_> = args.iter().map(|x| lower_expr(x, ctxt)).collect();
    let arg = ctxt.push_table();

    let is_function_ty = ctxt.alloc_blk();
    let is_no_function_ty = ctxt.alloc_blk();
    let is_class = ctxt.alloc_blk();
    let is_class_with_ctor = ctxt.alloc_blk();
    let is_class_finish = ctxt.alloc_blk();
    let err = ctxt.alloc_blk();
    let post = ctxt.alloc_blk();

    // if f["type"] == singletons["function"]: goto is_function_ty | is_no_function_ty
    let a = ctxt.push_index_str(f, "type");
    let b = ctxt.get_singleton("function");
    let cond = ctxt.branch_eq(a, b, is_function_ty, is_no_function_ty);

    ctxt.focus_blk(is_function_ty);
        lower_fn_type_call(f, &args[..], arg, ctxt);
        ctxt.push_goto(post);

    // if f["type"] == singletons["type"]: goto is_class | err
    ctxt.focus_blk(is_no_function_ty);
        let a = ctxt.push_index_str(f, "type");
        let b = ctxt.get_singleton("type");
        ctxt.branch_eq(a, b, is_class, err);

    ctxt.focus_blk(is_class);
        let u = ctxt.push_undef();
        let t = ctxt.build_value(u, f);
        let d = ctxt.push_index_str(f, "dict");
        let constr = ctxt.push_index_str(d, "__init__");
        ctxt.branch_undef(constr, is_class_finish, is_class_with_ctor);

    ctxt.focus_blk(is_class_with_ctor);
        args.insert(0, t);
        // we technically didn't check whether "constr" is even a function.
        lower_fn_type_call(constr, &args[..], arg, ctxt);
        ctxt.push_goto(is_class_finish);

    ctxt.focus_blk(is_class_finish);
        ctxt.push_store_str(arg, "ret", t);
        ctxt.push_goto(post);

    ctxt.focus_blk(err);
        ctxt.push_statement(Statement::Throw("can't call this thing!".to_string()));

    ctxt.focus_blk(post);
        ctxt.push_index_str(arg, "ret")
}

fn lower_assign(v: &str, val: Node, ctxt: &mut Ctxt) {
    let nn = find_namespace(v, ctxt);
    ctxt.push_store_str(nn, v, val);
}

fn lower_ast(ast: &[ASTStatement], ctxt: &mut Ctxt) {
    for stmt in ast {
        match stmt {
            ASTStatement::Expr(e) => { lower_expr(e, ctxt); },
            ASTStatement::Assign(ASTExpr::Var(v), rhs) => {
                let val = lower_expr(rhs, ctxt);
                lower_assign(v, val, ctxt);
            }
            ASTStatement::Assign(ASTExpr::Attribute(e, v), rhs) => {
                let e = lower_expr(e, ctxt);
                let d = ctxt.push_index_str(e, "dict");
                let val = lower_expr(rhs, ctxt);
                ctxt.push_store_str(d, v, val);
            },
            ASTStatement::If(cond, then) => {
                let cond = lower_expr(cond, ctxt);
                let b = ctxt.alloc_blk();
                let post = ctxt.alloc_blk();
                ctxt.push_if(cond, b, post);

                ctxt.focus_blk(b);
                lower_ast(then, ctxt);

                ctxt.push_goto(post);

                ctxt.focus_blk(post);
            }
            ASTStatement::While(cond, then) => {
                let pre = ctxt.alloc_blk();
                let b = ctxt.alloc_blk();
                let post = ctxt.alloc_blk();
                ctxt.fl_mut().loop_stack.push((post, pre));

                ctxt.push_goto(pre);
                ctxt.focus_blk(pre);
                let cond = lower_expr(cond, ctxt);

                ctxt.push_statement(Statement::If(cond, b, post));

                ctxt.focus_blk(b);
                lower_ast(then, ctxt);
                ctxt.push_goto(pre);

                ctxt.focus_blk(post);
                ctxt.fl_mut().loop_stack.pop();
            }
            ASTStatement::Def(name, args, body) => lower_def(name, args, body, stmt, ctxt),
            ASTStatement::Return(opt) => {
                let expr = opt.as_ref().unwrap_or(&ASTExpr::None);
                let val = lower_expr(expr, ctxt);
                ctxt.push_store_str(ctxt.fl().arg_node, "ret", val);
                ctxt.push_return();
            }
            ASTStatement::Pass => {} // do nothing
            ASTStatement::Break => {
                ctxt.push_goto(ctxt.fl().loop_stack.last().unwrap().0);
            }
            ASTStatement::Continue => {
                ctxt.push_goto(ctxt.fl().loop_stack.last().unwrap().1);
            }
            ASTStatement::Scope(..) => {} // scope is already handled in nameres
            ASTStatement::Class(name, _args, body) => {
                let dict = ctxt.push_table();
                let old_namespace = ctxt.fl_mut().namespace_node;
                // this temporarily overwrites the namespace node, so that local variables actually
                // write to the class instead.
                ctxt.fl_mut().namespace_node = dict;

                lower_ast(body, ctxt);
                let u = ctxt.push_undef();
                let type_ = ctxt.get_singleton("type");
                let val = ctxt.build_value_w_dict(u, type_, dict);

                ctxt.fl_mut().namespace_node = old_namespace;

                lower_assign(name, val, ctxt);
            }
            x => todo!("{:?}", x),
        }
    }
}

fn lower_def(name: &str, args: &[String], body: &[ASTStatement], stmt: &ASTStatement, ctxt: &mut Ctxt) {
    let i = new_fn(ctxt, |ctxt| {
        let argtable = ctxt.push_arg();
        ctxt.f_mut().lowering = Some(FnLowerCtxt {
            namespace_node: ctxt.push_table(),
            arg_node: argtable,
            global_node: ctxt.push_index_str(argtable, "scope_global"),
            singletons_node: ctxt.push_index_str(argtable, "singletons"),
            ast_ptr: stmt,
            loop_stack: Vec::new(),
        });

        // load args
        for (i, a) in args.iter().enumerate() {
            let i = ctxt.push_int(i as _);
            let val = ctxt.push_index(argtable, i);
            let nn = ctxt.fl().namespace_node;
            ctxt.push_store_str(nn, a, val);
        }

        lower_ast(body, ctxt);
        ctxt.push_return_none();
    });

    let function = ctxt.push_compute(Expr::Function(i));
    let function_t = ctxt.get_singleton("function");
    let val = ctxt.build_value(function, function_t);

    lower_assign(name, val, ctxt);
}

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

fn find_namespace(v: &str, ctxt: &mut Ctxt) -> Node {
    let k = (ctxt.fl().ast_ptr, v.to_string());
    match ctxt.nameres_tab.get(&k) {
        Some(VarPlace::Local) => ctxt.fl().namespace_node,
        _ => ctxt.fl().global_node,
    }
}
