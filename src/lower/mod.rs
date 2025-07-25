use crate::*;

mod nameres;
pub use nameres::*;

mod ctxt;
pub use ctxt::*;

pub fn lower(ast: &AST) -> String {
    let mut s = String::from("#\n");
    s.extend(lower_ast(ast).chars());
    s.extend(include_str!(concat!(env!("OUT_DIR"), "/concat.ir")).chars());

    s
}

fn lower_ast(ast: &AST) -> String {
    let nameres_tab = nameres(ast);
    let userstart = Symbol::new("userstart".to_string());
    let mut ctxt = Ctxt {
        stack: vec![FnCtxt {
            current_pid: userstart,
            lowering: Some(FnLowerCtxt {
                loop_stack: Vec::new(),
                ast_ptr: 0 as *const _,
            }),
        }],
        nameres_tab,
        procs: Map::new(),
    };

    ctxt.procs.insert(userstart, Vec::new());

    lower_body(&**ast, &mut ctxt);

    ctxt.push(format!("exit"));

    let mut s = String::new();
    for (pid, stmts) in ctxt.procs {
        s.push_str(&format!("proc {pid} {{\n"));
        for stmt in stmts {
            s.push_str(&format!("    {stmt};\n"));
        }
        s.push_str("}\n");
    }
    s
}

fn lower_body(stmts: &[ASTStatement], ctxt: &mut Ctxt) {
    for stmt in stmts {
        ctxt.push(format!("# {stmt:?}"));

        match stmt {
            ASTStatement::Expr(e) => {
                lower_expr(e, ctxt);
            },
            ASTStatement::Assign(ASTExpr::Var(var), val) => {
                let val = lower_expr(val, ctxt);
                lower_var_assign(&*var, val, ctxt)
            }
            ASTStatement::Assign(ASTExpr::Attribute(e, v), rhs) => {
                let e = lower_expr(e, ctxt);
                let rhs = lower_expr(rhs, ctxt);
                ctxt.push(format!("{e}.dict[\"{v}\"] = {rhs}"));
            },
            ASTStatement::Assign(ASTExpr::BinOp(ASTBinOpKind::Subscript, e, v), rhs) => {
                let e_setattr = Box::new(ASTExpr::Attribute(e.clone(), "__setitem__".to_string()));
                let real_stmt = ASTStatement::Expr(ASTExpr::FnCall(e_setattr, vec![(**v).clone(), rhs.clone()]));
                lower_body(&[real_stmt], ctxt);
            },
            ASTStatement::If(cond, then) => {
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
            ASTStatement::While(cond, body) => {
                let pre_pid = ctxt.alloc_blk();
                let body_pid = ctxt.alloc_blk();
                let post_pid = ctxt.alloc_blk();

                ctxt.fl_mut().loop_stack.push((post_pid, pre_pid));

                ctxt.push(format!("jmp {pre_pid}"));

                ctxt.focus_blk(pre_pid);
                    let cond = lower_expr(cond, ctxt);
                    let n = Symbol::new_fresh("whilecond".to_string());
                    ctxt.push(format!("%{n} = {{}}"));
                    ctxt.push(format!("%{n}[True] = {body_pid}"));
                    ctxt.push(format!("%{n}[False] = {post_pid}"));
                    ctxt.push(format!("jmp %{n}[{cond}.payload]"));

                ctxt.focus_blk(body_pid);
                    lower_body(body, ctxt);
                    ctxt.push(format!("jmp {pre_pid}"));

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
                let mut args: Vec<String> = args.iter().map(|x| lower_expr(x, ctxt)).collect();

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
                for except in excepts.iter().rev() {
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
                for except in excepts.iter() {
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
            ASTStatement::Raise(body) => {
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
                    ASTStatement::While(ASTExpr::Bool(true), body),
                ];
                let except = Except {
                    ty: None,
                    body: vec![ASTStatement::Pass],
                };
                let stmt = ASTStatement::Try(bod, vec![except]);
                lower_body(&[stmt], ctxt);
            },
            _ => todo!(),
        }
    }
}

fn lower_var_assign(var: &str, val: String, ctxt: &mut Ctxt) {
    let ns = find_namespace(var, ctxt);
    ctxt.push(format!("{ns}[\"{var}\"] = {val}"));
}

fn lower_expr(e: &ASTExpr, ctxt: &mut Ctxt) -> String {
    let out = match e {
        ASTExpr::FnCall(f, args) => {
            let f = lower_expr(f, ctxt);
            let suc = ctxt.alloc_blk();
            let arg = ctxt.alloc_irlocal("arg");
            ctxt.push(format!("{arg} = {{}}"));
            ctxt.push(format!("{arg}.f = {f}"));
            ctxt.push(format!("{arg}.suc = {suc}"));
            ctxt.push(format!("{arg}.args = {{}}"));
            for (i, a) in args.iter().enumerate() {
                let a = lower_expr(a, ctxt);
                ctxt.push(format!("{arg}.args[{i}] = {a}"));
            }
            ctxt.push(format!("@.arg = {arg}"));
            ctxt.push(format!("jmp py_call"));

            ctxt.focus_blk(suc);

            format!("@.ret")
        },
        ASTExpr::Var(v) => {
            let ns = find_namespace(v, ctxt);
            format!("{ns}[\"{v}\"]")
        },
        ASTExpr::Attribute(e, a) => {
            let e = lower_expr(e, ctxt);

            let suc = ctxt.alloc_blk();
            let arg = Symbol::new_fresh("arg");
            ctxt.push(format!("%{arg} = {{}}"));
            ctxt.push(format!("%{arg}.obj = {e}"));
            ctxt.push(format!("%{arg}.attr = \"{a}\""));
            ctxt.push(format!("%{arg}.suc = {suc}"));
            ctxt.push(format!("@.arg = %{arg}"));
            ctxt.push(format!("jmp py_attrlookup"));

            ctxt.focus_blk(suc);
                format!("@.ret")
        },
        ASTExpr::Str(s) => {
            let t = Symbol::new_fresh("strbox".to_string());
            ctxt.push(format!("%{t} = {{}}"));
            ctxt.push(format!("%{t}.type = @.singletons.str"));
            ctxt.push(format!("%{t}.payload = \"{s}\""));

            format!("%{t}")
        },
        ASTExpr::Int(i) => {
            let t = Symbol::new_fresh("intbox".to_string());
            ctxt.push(format!("%{t} = {{}}"));
            ctxt.push(format!("%{t}.type = @.singletons.int"));
            ctxt.push(format!("%{t}.payload = {i}"));

            format!("%{t}")
        },
        ASTExpr::Bool(b) => {
            match *b {
                true => format!("@.singletons.true"),
                false => format!("@.singletons.false"),
            }
        },
        ASTExpr::BinOp(kind, l, r) => {
            let l = lower_expr(l, ctxt);
            let r = lower_expr(r, ctxt);

            let l_op = op_attrs(*kind);
            let suc = ctxt.alloc_blk();
            let arg = Symbol::new_fresh("arg");
            ctxt.push(format!("%{arg} = {{}}"));
            ctxt.push(format!("%{arg}.suc = {suc}"));

            ctxt.push(format!("%{arg}.lhs = {l}"));
            ctxt.push(format!("%{arg}.rhs = {r}"));

            ctxt.push(format!("%{arg}.l_op = {{}}"));
            ctxt.push(format!("%{arg}.l_op.obj = @.singletons.str"));
            ctxt.push(format!("%{arg}.l_op.payload = \"{l_op}\""));
            ctxt.push(format!("@.arg = %{arg}"));

            ctxt.push(format!("jmp py_op"));

            ctxt.focus_blk(suc);
                format!("@.ret")
        },
        ASTExpr::None => format!("@.singletons.none"),
        ASTExpr::List(elems) => {
            let len = elems.len();
            let t = Symbol::new_fresh("listbox".to_string());
            ctxt.push(format!("%{t} = {{}}"));
            ctxt.push(format!("%{t}.type = @.singletons.list"));
            ctxt.push(format!("%{t}.dict = {{}}"));
            ctxt.push(format!("%{t}.payload = {{}}"));
            for (i, a) in elems.iter().enumerate() {
                let a = lower_expr(a, ctxt);
                ctxt.push(format!("%{t}.payload[{i}] = {a}"));
            }
            ctxt.push(format!("%{t}.length = {{}}"));
            ctxt.push(format!("%{t}.length.type = @.singletons.int"));
            ctxt.push(format!("%{t}.length.payload = {len}"));

            format!("%{t}")
        },
        _ => todo!("{:?}", e),
    };
    let irl = ctxt.alloc_irlocal("expr_val");
    ctxt.push(format!("{irl} = {out}"));
    format!("{irl}")
}

pub fn op_attrs(op: ASTBinOpKind) -> &'static str {
    match op {
        ASTBinOpKind::Plus => "__add__",
        ASTBinOpKind::Minus => "__sub__",
        ASTBinOpKind::Mul => "__mul__",
        ASTBinOpKind::Div => "__truediv__",
        ASTBinOpKind::Mod => "__mod__",
        ASTBinOpKind::Lt => "__lt__",
        ASTBinOpKind::Gt => "__gt__",
        ASTBinOpKind::Ge => "__ge__",
        ASTBinOpKind::Le => "__le__",
        ASTBinOpKind::IsEqual => "__eq__",
        ASTBinOpKind::IsNotEqual => "__ne__",
        ASTBinOpKind::Pow => "__pow__",
        ASTBinOpKind::Subscript => "__getitem__",
    }
}

pub fn find_namespace(v: &str, ctxt: &mut Ctxt) -> String {
    let k = (ctxt.fl().ast_ptr, v.to_string());
    match ctxt.nameres_tab.get(&k) {
        Some(VarPlace::Local) => format!("@.frame.pylocals"),
        _ => format!("@.globals"),
    }
}
