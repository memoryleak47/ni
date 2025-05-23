use crate::lower::*;

pub fn lower_expr(expr: &ASTExpr, ctxt: &mut Ctxt) -> Node {
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
        ASTExpr::Attribute(e, a) => lower_attribute(e, a, ctxt),
        _ => todo!("{:?}", expr),
    }
}

fn lower_attribute(e: &ASTExpr, a: &str, ctxt: &mut Ctxt) -> Node {
    let found = ctxt.alloc_blk();
    let not_found = ctxt.alloc_blk();
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
        lower_attribute_using_class(e, a, tmp, post, ctxt);
        ctxt.push_goto(post);

    ctxt.focus_blk(post);
        ctxt.push_index_str(tmp, "0")
}

// you evaluate `e.a` where you know that e does not contain a directly, so you check the class instead.
// writes the output to `tmp["0"]`, and jumps to `post`.
fn lower_attribute_using_class(e: Node, a: &str, tmp: Node, post: BlockId, ctxt: &mut Ctxt) {
    let found = ctxt.alloc_blk();
    let not_found = ctxt.alloc_blk();

    let t = ctxt.push_index_str(e, "type");
    let d = ctxt.push_index_str(t, "dict");
    let v = ctxt.push_index_str(d, a);
    ctxt.branch_undef(v, not_found, found);

    ctxt.focus_blk(found);
        ctxt.push_store_str(tmp, "0", v);
        ctxt.push_goto(post);

    ctxt.focus_blk(not_found);
        ctxt.push_statement(Statement::Throw("missing attribute!".to_string()));
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
