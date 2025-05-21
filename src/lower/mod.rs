use crate::*;

mod init;
pub use init::*;

mod ctxt;
pub use ctxt::*;

fn build_value(payload: Node, type_: Node, ctxt: &mut Ctxt) -> Node {
    let t = ctxt.push_compute(Expr::NewTable);
    ctxt.push_store_str(t, "type", type_);
    let dict = ctxt.push_compute(Expr::NewTable);
    ctxt.push_store_str(t, "payload", payload);
    ctxt.push_store_str(t, "dict", dict);
    t
}

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
            let nn = if let Some(VarPlace::Local) =
                ctxt.nameres_tab.get(&(ctxt.f().ast_ptr, v.to_string()))
            {
                ctxt.f().namespace_node
            } else {
                ctxt.f().global_node
            };
            ctxt.push_index_str(nn, v)
        }
        ASTExpr::FnCall(f, args) => lower_fn_call(&*f, args, ctxt),
        ASTExpr::Attribute(e, a) => {
            let e = lower_expr(e, ctxt);
            ctxt.push_index_str(e, a)
        },
        _ => todo!("{:?}", expr),
    }
}

fn lower_fn_call(f: &ASTExpr, args: &[ASTExpr], ctxt: &mut Ctxt) -> Node {
    let f = lower_expr(&f, ctxt);
    let arg = ctxt.push_table();

    let is_function_ty = ctxt.alloc_blk();
    let is_no_function_ty = ctxt.alloc_blk();
    let is_class = ctxt.alloc_blk();
    let err = ctxt.alloc_blk();
    let go = ctxt.alloc_blk();

    // where we store the function to call (under index "0").
    let tmp = ctxt.push_table();

    // if f["type"] == singletons["function"]: goto is_function_ty | is_no_function_ty
    let a = ctxt.push_index_str(f, "type");
    let b = ctxt.push_index_str(ctxt.f().singletons_node, "function");
    let cond = ctxt.push_eq(a, b);
    ctxt.push_if(cond, is_function_ty, is_no_function_ty);

    ctxt.focus_blk(is_function_ty);
    let f_payload = ctxt.push_index_str(f, "payload");
    ctxt.push_store_str(tmp, "0", f_payload);
    ctxt.push_goto(go);

    // if f["type"] == singletons["type"]: goto is_class | err
    ctxt.focus_blk(is_no_function_ty);
    let a = ctxt.push_index_str(f, "type");
    let b = ctxt.push_index_str(ctxt.f().singletons_node, "type");
    let cond = ctxt.push_eq(a, b);
    ctxt.push_if(cond, is_class, err);

    ctxt.focus_blk(is_class);
    let payload = ctxt.push_builtin("construct");
    ctxt.push_store_str(tmp, "0", payload);
    ctxt.push_goto(go);

    ctxt.focus_blk(err);
    ctxt.push_statement(Statement::Throw("can't call this thing!".to_string()));

    ctxt.focus_blk(go);

    // pass "scope_global" along.
    ctxt.push_store_str(arg, "scope_global", ctxt.f().global_node);

    // pass "singletons" along.
    ctxt.push_store_str(arg, "singletons", ctxt.f().singletons_node);

    for (i, a) in args.iter().enumerate() {
        let i = ctxt.push_compute(Expr::Int(i as _));
        let v = lower_expr(a, ctxt);
        ctxt.push_statement(Statement::Store(arg, i, v));
    }
    let f_payload = ctxt.push_index_str(tmp, "0");
    ctxt.push_statement(Statement::FnCall(f_payload, arg));
    ctxt.push_index_str(arg, "ret")
}

fn lower_assign(v: &str, val: Node, ctxt: &mut Ctxt) {
    let nn = if let Some(VarPlace::Local) = ctxt.nameres_tab.get(&(ctxt.f().ast_ptr, v.to_string()))
    {
        ctxt.f().namespace_node
    } else {
        ctxt.f().global_node
    };
    ctxt.push_store_str(nn, v, val);
}

fn lower_ast(ast: &AST, ctxt: &mut Ctxt) {
    for stmt in ast {
        match stmt {
            ASTStatement::Expr(e) => {
                lower_expr(e, ctxt);
            }
            ASTStatement::Assign(ASTExpr::Var(v), rhs) => {
                let val = lower_expr(rhs, ctxt);
                lower_assign(v, val, ctxt);
            }
            ASTStatement::Assign(ASTExpr::Attribute(e, v), rhs) => {
                let e = lower_expr(e, ctxt);
                let val = lower_expr(rhs, ctxt);
                ctxt.push_store_str(e, v, val);
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
                ctxt.f_mut().loop_stack.push((post, pre));

                ctxt.push_goto(pre);
                ctxt.focus_blk(pre);
                let cond = lower_expr(cond, ctxt);

                ctxt.push_statement(Statement::If(cond, b, post));

                ctxt.focus_blk(b);
                lower_ast(then, ctxt);
                ctxt.push_goto(pre);

                ctxt.focus_blk(post);
                ctxt.f_mut().loop_stack.pop();
            }
            ASTStatement::Def(name, args, body) => {
                let i = ctxt.ir.fns.len();
                ctxt.stack.push(FnCtxt::new(i, stmt as _));

                {
                    // add empty fn to IR
                    let mut blocks: HashMap<_, _> = Default::default();

                    // TODO unify this with the construction of the main function.
                    // this creates the namespace node.
                    blocks.insert(
                        0,
                        vec![
                            Statement::Compute(0, Expr::NewTable),
                            Statement::Compute(1, Expr::Arg),
                            Statement::Compute(2, Expr::Str("scope_global".to_string())),
                            Statement::Compute(3, Expr::Index(1, 2)),
                            Statement::Compute(4, Expr::Str("singletons".to_string())),
                            Statement::Compute(5, Expr::Index(1, 4)),
                        ],
                    );
                    let f = Function {
                        blocks,
                        start_block: 0,
                    };
                    ctxt.ir.fns.insert(i, f);
                }

                // load args
                let argtable = ctxt.push_arg();
                for (i, a) in args.iter().enumerate() {
                    let i = ctxt.push_int(i as _);
                    let val = ctxt.push_index(argtable, i);
                    let nn = ctxt.f().namespace_node;
                    ctxt.push_store_str(nn, a, val);
                }

                lower_ast(body, ctxt);
                ctxt.push_return();

                ctxt.stack.pop();

                let function = ctxt.push_compute(Expr::Function(i));
                let function_t = ctxt.push_index_str(ctxt.f().singletons_node, "function");
                let val = build_value(function, function_t, ctxt);

                lower_assign(name, val, ctxt);
            }
            ASTStatement::Return(opt) => {
                let expr = opt.as_ref().unwrap_or(&ASTExpr::None);
                let val = lower_expr(expr, ctxt);
                let argtable = ctxt.push_arg();
                ctxt.push_store_str(argtable, "ret", val);
                ctxt.push_return();
            }
            ASTStatement::Pass => {} // do nothing
            ASTStatement::Break => {
                ctxt.push_goto(ctxt.f().loop_stack.last().unwrap().0);
            }
            ASTStatement::Continue => {
                ctxt.push_goto(ctxt.f().loop_stack.last().unwrap().1);
            }
            ASTStatement::Scope(..) => {} // scope is already handled in nameres
            ASTStatement::Class(name, _args, body) => {
                // TODO: most stuff is missing here.

                lower_ast(body, ctxt);
                let val = ctxt.push_table();
                let type_ = ctxt.push_index_str(ctxt.f().singletons_node, "type");
                ctxt.push_store_str(val, "type", type_);
                lower_assign(name, val, ctxt);
            }
            x => todo!("{:?}", x),
        }
    }
}

pub fn lower(ast: &AST) -> IR {
    let mut ctxt = Ctxt::new(ast);

    add_builtins_and_singletons(&mut ctxt);
    lower_ast(ast, &mut ctxt);

    ctxt.push_return();
    ctxt.ir
}
