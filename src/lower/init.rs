use crate::lower::*;

fn add_print_builtin(ctxt: &mut Ctxt) {
    let print_fn = new_fn(ctxt, |ctxt| {
        let arg = ctxt.f().arg_node;
        let zero = ctxt.push_int(0);
        let first_arg = ctxt.push_index(arg, zero);
        let first_arg = ctxt.push_index_str(first_arg, "payload");
        ctxt.push_statement(Statement::Print(first_arg));
        ctxt.push_return_none();
    });
    ctxt.builtin_fns.insert("print".to_string(), print_fn);

    let print_f = ctxt.push_builtin("print");
    let function = ctxt.push_index_str(ctxt.f().singletons_node, "function");
    let print = ctxt.build_value(print_f, function);
    let nn = ctxt.fl().namespace_node;
    ctxt.push_store_str(nn, "print", print);
}

fn add_type_builtin(ctxt: &mut Ctxt) {
    let type_fn = new_fn(ctxt, |ctxt| {
        let arg = ctxt.f().arg_node;
        let zero = ctxt.push_int(0);
        let first_arg = ctxt.push_index(arg, zero);
        let ty = ctxt.push_index_str(first_arg, "type");
        ctxt.push_store_str(arg, "ret", ty);
        ctxt.push_return();
    });
    ctxt.builtin_fns.insert("type".to_string(), type_fn);

    let type_f = ctxt.push_builtin("type");
    let function = ctxt.push_index_str(ctxt.f().singletons_node, "function");
    let type_ = ctxt.build_value(type_f, function);
    let nn = ctxt.fl().namespace_node;
    ctxt.push_store_str(nn, "type", type_);
}

fn add_singletons(ctxt: &mut Ctxt) {
    let singleton = ctxt.f().singletons_node;

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

    let s = ctxt.push_str("None"); // hack: for now 'None' contains a string for correct printing.
    let none_ty = ctxt.get_singleton("NoneType");
    let none = ctxt.build_value(s, none_ty);
    ctxt.push_store_str(ctxt.f().singletons_node, "None", none);
}

static PRIM_TYPES: &[&'static str] = &[
    "function",
    "method",
    "str",
    "int",
    "float",
    "bool",
    "list",
    "NoneType",
];

static OPS: &[BinOpKind] =
    &[BinOpKind::Plus, BinOpKind::Minus, BinOpKind::Mul, BinOpKind::Div, BinOpKind::Mod, BinOpKind::Lt, BinOpKind::Gt, BinOpKind::Ge, BinOpKind::Le, BinOpKind::IsEqual, BinOpKind::IsNotEqual, BinOpKind::Pow];

fn add_ops(ctxt: &mut Ctxt) {
    for op in OPS {
        let fid = new_fn(ctxt, |ctxt| {
            let a = ctxt.push_index_int(ctxt.f().arg_node, 0);
            let a_ty = ctxt.push_index_str(a, "type");
            let a = ctxt.push_index_str(a, "payload");

            let b = ctxt.push_index_int(ctxt.f().arg_node, 1);
            let b = ctxt.push_index_str(b, "payload");

            let v = ctxt.push_compute(Expr::BinOp(*op, a, b));
            let v = ctxt.build_value(v, a_ty);
            ctxt.push_store_str(ctxt.f().arg_node, "ret", v);
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

fn add_list_getitem(ctxt: &mut Ctxt) {
    let f = new_fn(ctxt, |ctxt| {
        let arg = ctxt.f().arg_node;
        let list = ctxt.push_index_int(arg, 0);
        let index = ctxt.push_index_int(arg, 1);
        let list_payload = ctxt.push_index_str(list, "payload");
        let index_payload = ctxt.push_index_str(index, "payload");
        let result = ctxt.push_index(list_payload, index_payload);
        ctxt.push_store_str(arg, "ret", result);
        ctxt.push_return();
    });

    let function = ctxt.get_singleton("function");
    let f = ctxt.push_compute(Expr::Function(f));
    let getitem_fn = ctxt.build_value(f, function);
    let list_ty = ctxt.get_singleton("list");
    let list_ty_dict = ctxt.push_index_str(list_ty, "dict");
    ctxt.push_store_str(list_ty_dict, "__getitem__", getitem_fn);
}

fn add_list_setitem(ctxt: &mut Ctxt) {
    let f = new_fn(ctxt, |ctxt| {
        let arg = ctxt.f().arg_node;
        let list = ctxt.push_index_int(arg, 0);
        let index = ctxt.push_index_int(arg, 1);
        let val = ctxt.push_index_int(arg, 2);
        let list_payload = ctxt.push_index_str(list, "payload");
        let index_payload = ctxt.push_index_str(index, "payload");
        ctxt.push_statement(Statement::Store(list_payload, index_payload, val));
        ctxt.push_return_none();
    });

    let function = ctxt.get_singleton("function");
    let f = ctxt.push_compute(Expr::Function(f));
    let setitem_fn = ctxt.build_value(f, function);
    let list_ty = ctxt.get_singleton("list");
    let list_ty_dict = ctxt.push_index_str(list_ty, "dict");
    ctxt.push_store_str(list_ty_dict, "__setitem__", setitem_fn);
}

fn add_list_append(ctxt: &mut Ctxt) {
    let f = new_fn(ctxt, |ctxt| {
        let arg = ctxt.f().arg_node;
        let list = ctxt.push_index_int(arg, 0);
        let elem = ctxt.push_index_int(arg, 1);
        let list_payload = ctxt.push_index_str(list, "payload");
        let old_len = ctxt.push_index_str(list_payload, "len");
        let one = ctxt.push_int(1);
        let new_len = ctxt.push_compute(Expr::BinOp(BinOpKind::Plus, old_len, one));
        ctxt.push_store_str(list_payload, "len", new_len);
        ctxt.push_store(list_payload, old_len, elem);

        ctxt.push_return_none();
    });

    let function = ctxt.get_singleton("function");
    let f = ctxt.push_compute(Expr::Function(f));
    let fn_ = ctxt.build_value(f, function);
    let list_ty = ctxt.get_singleton("list");
    let list_ty_dict = ctxt.push_index_str(list_ty, "dict");
    ctxt.push_store_str(list_ty_dict, "append", fn_);
}

pub fn add_builtins_and_singletons(ctxt: &mut Ctxt) {
    add_singletons(ctxt);
    add_print_builtin(ctxt);
    add_type_builtin(ctxt);
    add_list_getitem(ctxt);
    add_list_setitem(ctxt);
    add_list_append(ctxt);

    add_ops(ctxt);
    for ty in PRIM_TYPES.iter().copied().chain(std::iter::once("type")) {
        let ty = ctxt.get_singleton(ty);
        add_ops_to_type(ty, ctxt);
    }
}
