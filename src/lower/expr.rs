use crate::lower::*;

pub fn lower_expr(expr: &ASTExpr, ctxt: &mut Ctxt) -> Node {
    if let Some(x) = lower_primitive(expr, ctxt) { return x; }

    match expr {
        ASTExpr::BinOp(op, lhs, rhs) => lower_binop(*op, lhs, rhs, ctxt),
        ASTExpr::Var(v) => {
            let nn = find_namespace(v, ctxt);
            ctxt.push_index_str(nn, v)
        }
        ASTExpr::FnCall(f, args) => lower_fn_call(&*f, args, ctxt),
        ASTExpr::Attribute(e, a) => lower_attribute(e, a, ctxt),
        ASTExpr::List(l) => {
            let t = ctxt.push_table();
            let ty = ctxt.get_singleton("list");
            let len_int = ctxt.push_int(l.len() as _);
            ctxt.push_store_str(t, "len", len_int);
            for (i, x) in l.iter().enumerate() {
                let x = lower_expr(x, ctxt);
                ctxt.push_store_int(t, i, x);
            }
            ctxt.build_value(t, ty)
        },
        _ => todo!("{:?}", expr),
    }
}

pub fn op_attrs(op: BinOpKind) -> &'static str {
    match op {
        BinOpKind::Plus => "__add__",
        BinOpKind::Minus => "__sub__",
        BinOpKind::Mul => "__mul__",
        BinOpKind::Div => "__truediv__",
        BinOpKind::Mod => "__mod__",
        BinOpKind::Lt => "__lt__",
        BinOpKind::Gt => "__gt__",
        BinOpKind::Ge => "__ge__",
        BinOpKind::Le => "__le__",
        BinOpKind::IsEqual => "__eq__",
        BinOpKind::IsNotEqual => "__ne__",
        BinOpKind::Pow => "__pow__",
        BinOpKind::Subscript => "__getitem__",
    }
}

fn lower_binop(op: BinOpKind, lhs: &ASTExpr, rhs: &ASTExpr, ctxt: &mut Ctxt) -> Node {
    let attr = op_attrs(op);
    let lhs = lower_expr(lhs, ctxt);
    let rhs = lower_expr(rhs, ctxt);

    // python doesn't check `lhs.attr`, but rather `type(lhs).attr` directly!
    let lhs_ty = ctxt.push_index_str(lhs, "type");
    let lhs_dict = ctxt.push_index_str(lhs_ty, "dict");
    let op = ctxt.push_index_str(lhs_dict, attr);
    let arg = ctxt.push_table();
    lower_fn_type_call(op, &[lhs, rhs], arg, ctxt);
    ctxt.push_index_str(arg, "ret")
}

fn lower_primitive(e: &ASTExpr, ctxt: &mut Ctxt) -> Option<Node> {
    Some(match e {
        ASTExpr::None => ctxt.get_singleton("None"),
        ASTExpr::Int(i) => {
            let i = ctxt.push_int(*i);
            let ty = ctxt.get_singleton("int");
            ctxt.build_value(i, ty)
        },
        ASTExpr::Bool(b) => {
            let b = ctxt.push_bool(*b);
            let ty = ctxt.get_singleton("bool");
            ctxt.build_value(b, ty)
        },
        ASTExpr::Str(s) => {
            let s = ctxt.push_str(s);
            let ty = ctxt.get_singleton("str");
            ctxt.build_value(s, ty)
        },
        _ => return None,
    })
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
    let pre_loop = ctxt.alloc_blk();
    let in_loop = ctxt.alloc_blk();
    let post_loop = ctxt.alloc_blk(); // reached only when the attr was found nowhere.

    let found = ctxt.alloc_blk();
    let found_is_fn = ctxt.alloc_blk();
    let found_is_no_fn = ctxt.alloc_blk();
    let not_found = ctxt.alloc_blk();

    let i = ctxt.push_table();
    let zero = ctxt.push_int(0);
    let one = ctxt.push_int(1);
    ctxt.push_store_str(i, "0", zero);

    let t = ctxt.push_index_str(e, "type");
    let mro = ctxt.push_index_str(t, "mro");
    let mro_list = ctxt.push_index_str(mro, "payload");
    let mro_len = ctxt.push_index_str(mro_list, "len");
    ctxt.push_goto(pre_loop);

    ctxt.focus_blk(pre_loop);
        let a_ = ctxt.push_index_str(i, "0");
        let b_ = mro_len;
        let cond = ctxt.push_compute(Expr::BinOp(BinOpKind::Lt, a_, b_));
        ctxt.push_if(cond, in_loop, post_loop);

    ctxt.focus_blk(in_loop);
        let i_val = ctxt.push_index_str(i, "0");
        let super_ty = ctxt.push_index(mro_list, i_val);
        let d = ctxt.push_index_str(super_ty, "dict");
        let v = ctxt.push_index_str(d, a);
        ctxt.branch_undef(v, not_found, found);

    ctxt.focus_blk(found);
        let v_t = ctxt.push_index_str(v, "type");
        let f = ctxt.get_singleton("function");
        ctxt.branch_eq(v_t, f, found_is_fn, found_is_no_fn);

    ctxt.focus_blk(found_is_fn); // return method object here:
        let method_ty = ctxt.get_singleton("method");
        let v_pay = ctxt.push_index_str(v, "payload");
        let method_obj = ctxt.build_value(v_pay, method_ty);
        ctxt.push_store_str(method_obj, "self", e);
        ctxt.push_store_str(tmp, "0", method_obj);
        ctxt.push_goto(post);

    ctxt.focus_blk(found_is_no_fn);
        ctxt.push_store_str(tmp, "0", v);
        ctxt.push_goto(post);

    ctxt.focus_blk(not_found);
        let i_val = ctxt.push_index_str(i, "0");
        let i1_val = ctxt.push_compute(Expr::BinOp(BinOpKind::Plus, i_val, one));
        ctxt.push_store_str(i, "0", i1_val); // i = i+1
        ctxt.push_goto(pre_loop);

    ctxt.focus_blk(post_loop);
        ctxt.push_statement(Statement::Throw("missing attribute!".to_string()));
}

// lower a function call, where you know that f is of type function.
// arg is empty thus far.
fn lower_fn_type_call(f: Node, args: &[Node], arg: Node, ctxt: &mut Ctxt) {
    ctxt.push_store_str(arg, "scope_global", ctxt.fl().global_node);
    ctxt.push_store_str(arg, "singletons", ctxt.f().singletons_node);

    for (i, a) in args.iter().enumerate() {
        ctxt.push_store_int(arg, i, *a);
    }

    let f_payload = ctxt.push_index_str(f, "payload");
    ctxt.push_statement(Statement::FnCall(f_payload, arg));
}

fn lower_fn_call(f: &ASTExpr, args: &[ASTExpr], ctxt: &mut Ctxt) -> Node {
    let f = lower_expr(&f, ctxt);
    let args: Vec<_> = args.iter().map(|x| lower_expr(x, ctxt)).collect();
    let arg = ctxt.push_table();

    let is_function_ty = ctxt.alloc_blk();
    let is_no_function_ty = ctxt.alloc_blk();
    let is_class = ctxt.alloc_blk();
    let is_class_with_ctor = ctxt.alloc_blk();
    let is_class_finish = ctxt.alloc_blk();
    let check_is_method = ctxt.alloc_blk();
    let is_method = ctxt.alloc_blk();
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

        ctxt.branch_eq(a, b, is_class, check_is_method);

    ctxt.focus_blk(is_class);
        let u = ctxt.push_undef();
        let t = ctxt.build_value(u, f);
        let d = ctxt.push_index_str(f, "dict");
        let constr = ctxt.push_index_str(d, "__init__");
        ctxt.branch_undef(constr, is_class_finish, is_class_with_ctor);

    ctxt.focus_blk(is_class_with_ctor);
        let local_args: Vec<Node> = std::iter::once(t).chain(args.iter().copied()).collect();
        // we technically didn't check whether "constr" is even a function.
        lower_fn_type_call(constr, &local_args[..], arg, ctxt);
        ctxt.push_goto(is_class_finish);

    ctxt.focus_blk(is_class_finish);
        ctxt.push_store_str(arg, "ret", t);
        ctxt.push_goto(post);

    ctxt.focus_blk(check_is_method);
        let a = ctxt.push_index_str(f, "type");
        let b = ctxt.get_singleton("method");
        ctxt.branch_eq(a, b, is_method, err);

    ctxt.focus_blk(is_method);
        let slf = ctxt.push_index_str(f, "self");
        let local_args: Vec<Node> = std::iter::once(slf).chain(args.iter().copied()).collect();
        lower_fn_type_call(f, &local_args[..], arg, ctxt);
        ctxt.push_goto(post);

    ctxt.focus_blk(err);
        ctxt.push_statement(Statement::Throw("can't call this thing!".to_string()));

    ctxt.focus_blk(post);
        ctxt.push_index_str(arg, "ret")
}
