use crate::lower::*;

fn add_print_builtin(ctxt: &mut Ctxt) {
    let print_fn = new_fn(ctxt, |ctxt| {
        let arg = ctxt.push_arg();
        let zero = ctxt.push_int(0);
        let first_arg = ctxt.push_index(arg, zero);
        ctxt.push_statement(Statement::Print(first_arg));
        let none = ctxt.push_none();
        ctxt.push_store_str(arg, "ret", none);
        ctxt.push_return();
    });
    ctxt.builtin_fns.insert("print".to_string(), print_fn);

    let print_f = ctxt.push_builtin("print");
    let function = ctxt.push_index_str(ctxt.fl().singletons_node, "function");
    let print = build_value(print_f, function, ctxt);
    let nn = ctxt.fl().namespace_node;
    ctxt.push_store_str(nn, "print", print);
}

fn add_construct_builtin(ctxt: &mut Ctxt) {
    let f = new_fn(ctxt, |ctxt| {
        let arg = ctxt.push_arg();
        let t = ctxt.push_table();
        ctxt.push_store_str(arg, "ret", t);
        ctxt.push_return();
    });

    ctxt.builtin_fns.insert("construct".to_string(), f);
}

fn add_singletons(ctxt: &mut Ctxt) {
    let singleton = ctxt.push_compute(Expr::NewTable);
    ctxt.fl_mut().singletons_node = singleton;

    let type_ = ctxt.push_compute(Expr::NewTable);
    let type_str = ctxt.push_str("type");
    ctxt.push_store(singleton, type_str, type_);

    // type is of type `type`.
    ctxt.push_store(type_, type_str, type_);

    let mut add_primitive_type = |name| {
        let tab = ctxt.push_compute(Expr::NewTable);
        let name_str = ctxt.push_str(name);
        ctxt.push_store(tab, type_str, type_);
        ctxt.push_store(singleton, name_str, tab);
    };

    add_primitive_type("function");
    add_primitive_type("str");
    add_primitive_type("int");
    add_primitive_type("float");
    add_primitive_type("bool");
    add_primitive_type("NoneType");
}

pub fn add_builtins_and_singletons(ctxt: &mut Ctxt) {
    add_singletons(ctxt);
    add_print_builtin(ctxt);
    add_construct_builtin(ctxt);
}
