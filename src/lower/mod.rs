use crate::*;

mod nameres;
pub use nameres::*;

mod ctxt;
pub use ctxt::*;

pub fn lower(ast: &AST) -> String {
    let mut s = lower_ast(ast);
    s.extend(include_str!("../sem/types.ir").chars());
    s.extend(include_str!("../sem/init.ir").chars());
    s.extend(include_str!("../sem/op.ir").chars());
    s.extend(include_str!("../sem/attr.ir").chars());
    s
}

fn lower_ast(ast: &AST) -> String {
    let nameres_tab = nameres(ast);
    let userstart = Symbol::new("userstart".to_string());
    let mut ctxt = Ctxt {
        stack: vec![FnCtxt {
            current_pid: userstart,
            lowering: Some(FnLowerCtxt {
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
    for x in stmts {
        lower_stmt(x, ctxt);
    }
}

fn lower_var_assign(var: &str, val: String, ctxt: &mut Ctxt) {
    let ns = find_namespace(var, ctxt);
    ctxt.push(format!("{ns}[\"{var}\"] = {val}"));
}

fn lower_stmt(stmt: &ASTStatement, ctxt: &mut Ctxt) {
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
        },
        ASTStatement::Def(name, args, body) => {
            let pid = Symbol::new_fresh(name.to_string());
            ctxt.procs.insert(pid, Vec::new());
            ctxt.stack.push(FnCtxt {
                current_pid: pid,
                lowering: Some(FnLowerCtxt {
                    ast_ptr: stmt as *const _,
                }),
            });

            // might be overwritten by something else in the meantime.
            ctxt.push(format!("@.frame.retval.v = @.singletons.none"));

            for (i, a) in args.iter().enumerate() {
                ctxt.push(format!("@.frame.pylocals[\"{a}\"] = @.frame.arg[{i}]"));
            }

            lower_body(body, ctxt);

            let frame = Symbol::new_fresh("frame");
            ctxt.push(format!("%{frame} = @.frame"));
            ctxt.push(format!("@.frame = %{frame}.parent"));
            ctxt.push(format!("jmp %{frame}.retpid"));

            ctxt.stack.pop();

            
            let val = Symbol::new_fresh("functionbox");
            ctxt.push(format!("%{val} = {{}}"));
            ctxt.push(format!("%{val}.type = @.singletons.function"));
            ctxt.push(format!("%{val}.payload = {pid}"));

            lower_var_assign(name, format!("%{val}"), ctxt);
        },
        _ => todo!(),
    }
}

fn lower_expr(e: &ASTExpr, ctxt: &mut Ctxt) -> String {
    match e {
        ASTExpr::FnCall(f, args) => {
            let f = lower_expr(f, ctxt);
            let suc = ctxt.alloc_blk();
            let new_f = Symbol::new_fresh("new_frame");
            ctxt.push(format!("%{new_f} = {{}}"));
            ctxt.push(format!("%{new_f}.parent = @.frame"));
            ctxt.push(format!("%{new_f}.arg = {{}}"));
            ctxt.push(format!("%{new_f}.retval = {{}}"));
            ctxt.push(format!("%{new_f}.retpid = {suc}"));
            ctxt.push(format!("%{new_f}.pylocals = {{}}"));
            ctxt.push(format!("%{new_f}.irlocals = {{}}"));
            for (i, a) in args.iter().enumerate() {
                let a = lower_expr(a, ctxt);
                ctxt.push(format!("%{new_f}.arg[{i}] = {a}"));
            }
            let ff = Symbol::new_fresh("fn");
            ctxt.push(format!("%{ff} = {f}"));
            ctxt.push(format!("@.frame = %{new_f}"));
            ctxt.push(format!("jmp %{ff}.payload"));

            ctxt.focus_blk(suc);

            format!("%{new_f}.retval.v") // TODO the node new_f will be out of scope
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
            let post = ctxt.alloc_blk();
            let new_f = Symbol::new_fresh("new_frame");
            ctxt.push(format!("%{new_f} = {{}}"));
            ctxt.push(format!("%{new_f}.parent = @.frame"));
            ctxt.push(format!("%{new_f}.arg = {{}}"));
            ctxt.push(format!("%{new_f}.arg.lhs = {l}"));
            ctxt.push(format!("%{new_f}.arg.rhs = {r}"));
            ctxt.push(format!("%{new_f}.arg.l_op = {l_op}"));
            ctxt.push(format!("@.frame.irlocals.opret = {{}}"));
            ctxt.push(format!("%{new_f}.retval = @.frame.irlocals.opret"));
            ctxt.push(format!("%{new_f}.retpid = {post}"));
            ctxt.push(format!("%{new_f}.pylocals = {{}}"));
            ctxt.push(format!("%{new_f}.irlocals = {{}}"));
            ctxt.push(format!("@.frame = %{new_f}"));
            ctxt.push(format!("jmp op"));

            ctxt.focus_blk(post);
                format!("@.frame.irlocals.opret")
        },
        _ => todo!("{:?}", e),
    }
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
