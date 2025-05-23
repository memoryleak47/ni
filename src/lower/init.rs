use crate::lower::*;

fn add_print_builtin(ctxt: &mut Ctxt) {
    let print_fn = new_fn(ctxt, |ctxt| {
        let arg = ctxt.push_arg();
        let zero = ctxt.push_int(0);
        let first_arg = ctxt.push_index(arg, zero);
        let first_arg = ctxt.push_index_str(first_arg, "payload");
        ctxt.push_statement(Statement::Print(first_arg));
        let none = ctxt.push_none();
        ctxt.push_store_str(arg, "ret", none);
        ctxt.push_return();
    });
    ctxt.builtin_fns.insert("print".to_string(), print_fn);

    let print_f = ctxt.push_builtin("print");
    let function = ctxt.push_index_str(ctxt.fl().singletons_node, "function");
    let print = ctxt.build_value(print_f, function);
    let nn = ctxt.fl().namespace_node;
    ctxt.push_store_str(nn, "print", print);
}

fn add_type_builtin(ctxt: &mut Ctxt) {
    let type_fn = new_fn(ctxt, |ctxt| {
        let arg = ctxt.push_arg();
        let zero = ctxt.push_int(0);
        let first_arg = ctxt.push_index(arg, zero);
        let ty = ctxt.push_index_str(first_arg, "type");
        ctxt.push_store_str(arg, "ret", ty);
        ctxt.push_return();
    });
    ctxt.builtin_fns.insert("type".to_string(), type_fn);

    let type_f = ctxt.push_builtin("type");
    let function = ctxt.push_index_str(ctxt.fl().singletons_node, "function");
    let type_ = ctxt.build_value(type_f, function);
    let nn = ctxt.fl().namespace_node;
    ctxt.push_store_str(nn, "type", type_);
}

fn add_singletons(ctxt: &mut Ctxt) {
    let singleton = ctxt.push_compute(Expr::NewTable);
    ctxt.fl_mut().singletons_node = singleton;

    let type_ = ctxt.push_table();
    ctxt.push_store_str(singleton, "type", type_);

    // type is of type `type`.
    ctxt.push_store_str(type_, "type", type_);
    let dict = ctxt.push_table();
    ctxt.push_store_str(type_, "dict", dict);

    for name in PRIM_TYPES {
        let tab = ctxt.push_table();
        let dict = ctxt.push_table();
        ctxt.push_store_str(tab, "type", type_);
        ctxt.push_store_str(tab, "dict", dict);
        ctxt.push_store_str(singleton, name, tab);
    }
}

static PRIM_TYPES: &[&'static str] = &[
    "function",
    "method",
    "str",
    "int",
    "float",
    "bool",
    "NoneType",
];

static OPS: &[BinOpKind] =
    &[BinOpKind::Plus, BinOpKind::Minus, BinOpKind::Mul, BinOpKind::Div, BinOpKind::Mod, BinOpKind::Lt, BinOpKind::Gt, BinOpKind::Ge, BinOpKind::Le, BinOpKind::IsEqual, BinOpKind::IsNotEqual, BinOpKind::Pow];

fn add_ops(ctxt: &mut Ctxt) {
    for op in OPS {
        let fid = new_fn(ctxt, |ctxt| {
            let arg = ctxt.push_arg();
            let a = ctxt.push_index_int(arg, 0);
            let a_ty = ctxt.push_index_str(a, "type");
            let a = ctxt.push_index_str(a, "payload");

            let b = ctxt.push_index_int(arg, 1);
            let b = ctxt.push_index_str(b, "payload");

            let v = ctxt.push_compute(Expr::BinOp(*op, a, b));
            let v = ctxt.build_value(v, a_ty);
            ctxt.push_store_str(arg, "ret", v);
            ctxt.push_return();
        });

        ctxt.builtin_fns.insert(op_attrs(*op).to_string(), fid);
    }
}

fn add_ops_to_type(ty: Node, ctxt: &mut Ctxt) {
    for op in OPS {
        let fid = ctxt.builtin_fns[op_attrs(*op)];
        let fn_ty = ctxt.get_singleton("function");
        let int_dict = ctxt.push_index_str(ty, "dict");
        let f = ctxt.push_compute(Expr::Function(fid));
        let f_val = ctxt.build_value(f, fn_ty);
        ctxt.push_store_str(int_dict, op_attrs(*op), f_val);
    }
}

pub fn add_builtins_and_singletons(ctxt: &mut Ctxt) {
    add_singletons(ctxt);
    add_print_builtin(ctxt);
    add_type_builtin(ctxt);

    add_ops(ctxt);
    for ty in PRIM_TYPES.iter().copied().chain(std::iter::once("type")) {
        let ty = ctxt.get_singleton(ty);
        add_ops_to_type(ty, ctxt);
    }
}
