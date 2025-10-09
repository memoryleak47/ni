use crate::lower::*;

pub fn lower_body(stmts: &[ASTStatement], ctxt: &mut Ctxt) {
    for stmt in stmts {
        ctxt.push(format!("# {stmt:?}"));

        match stmt {
            ASTStatement::Expr(e) => {
                lower_expr(e, ctxt);
            },
            ASTStatement::Assign(lhs, rhs) => {
                let lhs = lower_pexpr(lhs, ctxt);
                let rhs = lower_expr(rhs, ctxt);
                pexpr_store(&lhs, rhs, ctxt);
            },
            ASTStatement::AugAssign(lhs, op, rhs) => {
                let lhs = lower_pexpr(lhs, ctxt);
                let lhs_v = pexpr_load(&lhs, ctxt);
                let rhs_v = lower_expr(rhs, ctxt);

                // TODO try `__iadd__` etc. first.
                let op = aug_op_attr_fallbacks(*op);
                let out = lower_binop(op, lhs_v, rhs_v, ctxt);

                pexpr_store(&lhs, out, ctxt);
            },
            ASTStatement::If(cond, then, else_) => {
                assert!(else_.is_none(), "TODO: handle else");

                let cond = lower_expr(cond, ctxt);
                let n = Symbol::new_fresh("ifcond".to_string());
                let then_pid = ctxt.alloc_blk();
                let post_pid = ctxt.alloc_blk();
                ctxt.push(format!("%{n} = {{}}"));
                ctxt.push(format!("%{n}[True] = {then_pid}"));
                ctxt.push(format!("%{n}[False] = {post_pid}"));
                ctxt.push(format!("jmp %{n}[{cond}.payload]"));

                ctxt.focus_blk(then_pid);
                    lower_body(then, ctxt);
                    ctxt.push(format!("jmp {post_pid}"));

                ctxt.focus_blk(post_pid);
            },
            ASTStatement::While(cond, body, else_) => {
                let pre_pid = ctxt.alloc_blk();
                let body_pid = ctxt.alloc_blk();
                let else_pid = ctxt.alloc_blk();
                let post_pid = ctxt.alloc_blk();

                ctxt.fl_mut().loop_stack.push((post_pid, pre_pid));

                ctxt.push(format!("jmp {pre_pid}"));

                ctxt.focus_blk(pre_pid);
                    let cond = lower_expr(cond, ctxt);
                    let n = Symbol::new_fresh("whilecond".to_string());
                    ctxt.push(format!("%{n} = {{}}"));
                    ctxt.push(format!("%{n}[True] = {body_pid}"));
                    ctxt.push(format!("%{n}[False] = {else_pid}"));
                    ctxt.push(format!("jmp %{n}[{cond}.payload]"));

                ctxt.focus_blk(body_pid);
                    lower_body(body, ctxt);
                    ctxt.push(format!("jmp {pre_pid}"));

                ctxt.focus_blk(else_pid);
                    if let Some(else_) = else_ {
                        lower_body(else_, ctxt);
                    }
                    ctxt.push(format!("jmp {post_pid}"));

                ctxt.focus_blk(post_pid);

                ctxt.fl_mut().loop_stack.pop();
            },
            ASTStatement::Break => {
                let pid = ctxt.fl_mut().loop_stack.last().unwrap().0;
                ctxt.push(format!("jmp {pid}"));
                return;
            },
            ASTStatement::Continue => {
                let pid = ctxt.fl_mut().loop_stack.last().unwrap().1;
                ctxt.push(format!("jmp {pid}"));
                return;
            },
            ASTStatement::Def(name, args, body) => {
                // add a return, incase it's missing.
                let mut body: Vec<ASTStatement> = body.iter().cloned().collect();
                body.push(ASTStatement::Return(None));
                let body = &*body;

                let pid = Symbol::new_fresh(format!("f_{name}"));
                ctxt.procs.insert(pid, Vec::new());
                ctxt.stack.push(FnCtxt {
                    current_pid: pid,
                    lowering: Some(FnLowerCtxt {
                        loop_stack: Vec::new(),
                        ast_ptr: stmt as *const _,
                    }),
                });

                for (i, a) in args.iter().enumerate() {
                    ctxt.push(format!("@.frame.pylocals[\"{a}\"] = @.arg[{i}]"));
                }

                lower_body(body, ctxt);

                ctxt.stack.pop();

                let val = Symbol::new_fresh("functionbox");
                ctxt.push(format!("%{val} = {{}}"));
                ctxt.push(format!("%{val}.type = @.singletons.function"));
                ctxt.push(format!("%{val}.dict = {{}}"));
                ctxt.push(format!("%{val}.payload = {pid}"));

                lower_var_assign(name, format!("%{val}"), ctxt);
            },
            ASTStatement::Return(obj) => {
                let mut n = format!("@.singletons.none");
                if let Some(o) = obj {
                    n = lower_expr(o, ctxt);
                }
                ctxt.push(format!("@.ret = {n}"));
                ctxt.push(format!("jmp pop_stack"));
                return;
            },
            ASTStatement::Pass => {},
            ASTStatement::Scope(..) => {}, // Scope is handled in nameres.
            ASTStatement::Class(name, args, body) => {
                let args: Vec<String> = args.iter().map(|x| lower_expr(x, ctxt)).collect();

                let old_ptr = ctxt.fl().ast_ptr;
                ctxt.fl_mut().ast_ptr = stmt as _;

                let dict = ctxt.alloc_irlocal("class_dict");
                let old_namespace = ctxt.alloc_irlocal("old_namespace");
                ctxt.push(format!("{old_namespace} = @.frame.pylocals"));
                ctxt.push(format!("{dict} = {{}}"));
                ctxt.push(format!("@.frame.pylocals = {dict}"));

                lower_body(body, ctxt);

                let cl = ctxt.alloc_irlocal("class_obj");
                ctxt.push(format!("{cl} = {{}}"));
                ctxt.push(format!("{cl}.type = @.singletons.type"));
                ctxt.push(format!("{cl}.dict = {dict}"));

                let suc = ctxt.alloc_blk();
                ctxt.push(format!("@.arg = {{}}"));
                ctxt.push(format!("@.arg.obj = {cl}"));
                ctxt.push(format!("@.arg.parents = {{}}"));
                for (i, a) in args.iter().enumerate() {
                    ctxt.push(format!("@.arg.parents[{i}] = {a}"));
                }
                ctxt.push(format!("@.arg.suc = {suc}"));
                ctxt.push(format!("jmp add_mro"));

                ctxt.focus_blk(suc);
                ctxt.push(format!("@.frame.pylocals = {old_namespace}"));

                lower_var_assign(name, format!("{cl}"), ctxt);
                ctxt.fl_mut().ast_ptr = old_ptr;
            },
            ASTStatement::Try(body, excepts) => {
                let suc = ctxt.alloc_blk();

                let mut pids = Vec::new();

                // We push to the handler stack in reverted order, as the first `except` should be at the top of the stack.
                for _except in excepts.iter().rev() {
                    let h = ctxt.alloc_irlocal("handler");
                    let except_pid = ctxt.alloc_blk();
                    pids.push(except_pid);

                    // push handler stack
                    ctxt.push(format!("{h} = {{}}"));
                    ctxt.push(format!("{h}.parent = @.handler"));
                    ctxt.push(format!("{h}.frame = @.frame"));
                    ctxt.push(format!("{h}.pid = {except_pid}"));
                    ctxt.push(format!("@.handler = {h}"));
                }

                lower_body(body, ctxt);

                // pop handler stack
                for _except in excepts.iter() {
                    ctxt.push(format!("@.handler = @.handler.parent"));
                }

                ctxt.push(format!("jmp {suc}"));

                for (pid, except) in pids.iter().zip(excepts.iter().rev()) {
                    ctxt.focus_blk(*pid);
                        lower_body(&except.body, ctxt);
                        ctxt.push(format!("jmp {suc}"));
                }

                ctxt.focus_blk(suc);
            },
            ASTStatement::Raise(_body) => {
                ctxt.push(format!("jmp raise"))
            }
            ASTStatement::For(v, expr, body) => {
                // The $ sign prevents collisions with user-defined variables.
                let hv = Symbol::new_fresh("$hiddenvar");

                let expr = ASTExpr::FnCall(Box::new(
                    ASTExpr::Attribute(Box::new(expr.clone()), String::from("__iter__"))),
                    vec![]);
                let mut body = body.clone();
                body.insert(0, ASTStatement::Assign(
                    ASTExpr::Var(v.to_string()),
                    ASTExpr::FnCall(
                        Box::new(ASTExpr::Attribute(
                            Box::new(ASTExpr::Var(hv.to_string())),
                            "__next__".to_string(),
                        )),
                   vec![]),
                ));
                let bod = vec![
                    ASTStatement::Assign(ASTExpr::Var(hv.to_string()), expr),
                    ASTStatement::While(ASTExpr::Bool(true), body, None),
                ];
                let except = Except {
                    ty: None,
                    body: vec![ASTStatement::Pass],
                };
                let stmt = ASTStatement::Try(bod, vec![except]);
                lower_body(&[stmt], ctxt);
            },
        }
    }
}

pub fn lower_var_assign(var: &str, val: String, ctxt: &mut Ctxt) {
    let ns = find_namespace(var, ctxt);
    ctxt.push(format!("{ns}[\"{var}\"] = {val}"));
}

pub fn find_namespace(v: &str, ctxt: &mut Ctxt) -> Lowered {
    let k = (ctxt.fl().ast_ptr, v.to_string());
    match ctxt.nameres_tab.get(&k) {
        Some(VarPlace::Local) => format!("@.frame.pylocals"),
        _ => format!("@.globals"),
    }
}

