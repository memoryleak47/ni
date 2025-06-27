use crate::*;

mod nameres;
pub use nameres::*;

mod ctxt;
pub use ctxt::*;

pub fn lower(ast: &AST) -> String {
    let mut s = lower_ast(ast);
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
                // TODO
                // add_mro(val, &args, ctxt);

                ctxt.push(format!("@.frame.pylocals = {old_namespace}"));

                lower_var_assign(name, format!("{cl}"), ctxt);
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
            let b = if *b { "True" } else { "False" };
            let t = Symbol::new_fresh("boolbox".to_string());
            ctxt.push(format!("%{t} = {{}}"));
            ctxt.push(format!("%{t}.type = @.singletons.bool"));
            ctxt.push(format!("%{t}.payload = {b}"));

            format!("%{t}")
        },
        ASTExpr::BinOp(kind, l, r) => {
            let l = lower_expr(l, ctxt);
            let r = lower_expr(r, ctxt);

            let l_op = op_attrs(*kind);
            let suc = ctxt.alloc_blk();
            let arg = Symbol::new_fresh("arg");
            ctxt.push(format!("%{arg} = {{}}"));
            ctxt.push(format!("%{arg}.f = py_op"));
            ctxt.push(format!("%{arg}.suc = {suc}"));
            ctxt.push(format!("%{arg}.farg = {{}}"));

            ctxt.push(format!("%{arg}.farg.lhs = {l}"));
            ctxt.push(format!("%{arg}.farg.rhs = {r}"));

            ctxt.push(format!("%{arg}.farg.l_op = {{}}"));
            ctxt.push(format!("%{arg}.farg.l_op.type = @.singletons.str"));
            ctxt.push(format!("%{arg}.farg.l_op.payload = \"{l_op}\""));
            ctxt.push(format!("@.arg = %{arg}"));

            ctxt.push(format!("jmp call_fn"));

            ctxt.focus_blk(suc);
                format!("@.ret")
        },
        ASTExpr::None => format!("@.singletons.none"),
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
