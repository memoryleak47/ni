use crate::lower::*;

pub fn lower_assign(v: &str, val: Node, ctxt: &mut Ctxt) {
    let nn = find_namespace(v, ctxt);
    ctxt.push_store_str(nn, v, val);
}

// lowers an expression and converts it to a bool.
fn lower_cond(e: &ASTExpr, ctxt: &mut Ctxt) -> Node {
    let e = lower_expr(e, ctxt);
    // so far we assume it's a bool!
    ctxt.push_index_str(e, "payload")
}

pub fn lower_ast(ast: &[ASTStatement], ctxt: &mut Ctxt) {
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
            ASTStatement::Assign(ASTExpr::BinOp(BinOpKind::Subscript, e, v), rhs) => {
                let e_setattr = Box::new(ASTExpr::Attribute(e.clone(), "__setitem__".to_string()));
                let real_stmt = ASTStatement::Expr(ASTExpr::FnCall(e_setattr, vec![(**v).clone(), rhs.clone()]));
                lower_ast(&[real_stmt], ctxt);
            },
            ASTStatement::If(cond, then) => {
                let cond = lower_cond(cond, ctxt);
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
                let cond = lower_cond(cond, ctxt);

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
            let val = ctxt.push_index_int(argtable, i);
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
