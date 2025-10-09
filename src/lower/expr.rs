use crate::lower::*;

pub fn lower_expr(e: &ASTExpr, ctxt: &mut Ctxt) -> Lowered {
    let out = match e {
        ASTExpr::FnCall(f, args) => {
            let f = lower_expr(f, ctxt);

            let mut args2 = Vec::new();
            for a in args.iter() {
                args2.push(lower_expr(a, ctxt));
            }

            lower_fn_call(f, args2, ctxt)
        },
        ASTExpr::Var(..) | ASTExpr::Attribute(..) | ASTExpr::BinOp(ASTBinOpKind::Subscript, ..) => {
            let e = lower_pexpr(e, ctxt);
            pexpr_load(&e, ctxt)
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
        ASTExpr::BinOp(op@(ASTBinOpKind::And | ASTBinOpKind::Or), l, r) => {
            let l = lower_expr(l, ctxt);

            let suc = ctxt.alloc_blk();
            let snd = ctxt.alloc_blk();

            let arg = ctxt.alloc_irlocal("arg");
            ctxt.push(format!("{arg} = {{}}"));
            ctxt.push(format!("{arg}.elem = {l}"));

            let jmptab = Symbol::new_fresh("jmptab");
            ctxt.push(format!("%{jmptab} = {{}}"));

            if let ASTBinOpKind::And = op {
                ctxt.push(format!("%{jmptab}[{l}.payload] = {suc}"));
                ctxt.push(format!("%{jmptab}[True] = {snd}"));
            } else {
                ctxt.push(format!("%{jmptab}[{l}.payload] = {snd}"));
                ctxt.push(format!("%{jmptab}[True] = {suc}"));
            }

            ctxt.push(format!("jmp %{jmptab}[{l}.payload]"));

            ctxt.focus_blk(snd);
                let r = lower_expr(r, ctxt);
                ctxt.push(format!("{arg}.elem = {r}"));
                ctxt.push(format!("jmp {suc}"));

            ctxt.focus_blk(suc);
                format!("{arg}.elem")
        },
        ASTExpr::BinOp(kind, l, r) => {
            let l = lower_expr(l, ctxt);
            let r = lower_expr(r, ctxt);
            lower_binop(*kind, l, r, ctxt)
        },
        ASTExpr::UnOp(ASTUnOpKind::Neg, e) => {
            let e = lower_expr(e, ctxt);

            let suc = ctxt.alloc_blk();
            let arg = Symbol::new_fresh("arg");
            ctxt.push(format!("%{arg} = {{}}"));
            ctxt.push(format!("%{arg}.suc = {suc}"));

            ctxt.push(format!("%{arg}.obj = {e}"));

            ctxt.push(format!("%{arg}.l_op = {{}}"));
            ctxt.push(format!("%{arg}.l_op.obj = @.singletons.str"));
            ctxt.push(format!("%{arg}.l_op.payload = \"__neg__\""));
            ctxt.push(format!("@.arg = %{arg}"));

            ctxt.push(format!("jmp py_unop"));

            ctxt.focus_blk(suc);
                format!("@.ret")
        },
        ASTExpr::None => format!("@.singletons.none"),
        ASTExpr::List(elems) => {
            let len = elems.len();
            let t = Symbol::new_fresh("listbox");
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
        ASTExpr::Slice(b) => {
            let (a, b, c) = &**b;
            let a = a.as_ref().map(|a| lower_expr(a, ctxt));
            let b = b.as_ref().map(|b| lower_expr(b, ctxt));
            let c = c.as_ref().map(|c| lower_expr(c, ctxt));

            let t = Symbol::new_fresh("slicebox");
            ctxt.push(format!("%{t} = {{}}"));
            ctxt.push(format!("%{t}.type = @.singletons.slice"));
            if let Some(a) = a {
                ctxt.push(format!("%{t}.start = {a}"));
            }
            if let Some(b) = b {
                ctxt.push(format!("%{t}.end = {b}"));
            }
            if let Some(c) = c {
                ctxt.push(format!("%{t}.step = {c}"));
            }

            format!("%{t}")
        },
    };
    let irl = ctxt.alloc_irlocal("expr_val");
    ctxt.push(format!("{irl} = {out}"));
    format!("{irl}")
}

pub fn lower_fn_call(f: Lowered, args: Vec<Lowered>, ctxt: &mut Ctxt) -> Lowered {
    let suc = ctxt.alloc_blk();
    let arg = ctxt.alloc_irlocal("arg");
    ctxt.push(format!("{arg} = {{}}"));
    ctxt.push(format!("{arg}.f = {f}"));
    ctxt.push(format!("{arg}.suc = {suc}"));
    ctxt.push(format!("{arg}.args = {{}}"));
    for (i, a) in args.iter().enumerate() {
        ctxt.push(format!("{arg}.args[{i}] = {a}"));
    }
    ctxt.push(format!("@.arg = {arg}"));
    ctxt.push(format!("jmp py_call"));

    ctxt.focus_blk(suc);

    format!("@.ret")
}


pub fn lower_binop(kind: ASTBinOpKind, l: Lowered, r: Lowered, ctxt: &mut Ctxt) -> Lowered {
    let l_op = op_attrs(kind);
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

    ctxt.push(format!("jmp py_binop"));

    ctxt.focus_blk(suc);
        format!("@.ret")
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

        ASTBinOpKind::And | ASTBinOpKind::Or => unreachable!(),
    }
}

pub fn aug_op_attrs(op: ASTAugOpKind) -> &'static str {
    match op {
        ASTAugOpKind::PlusEq => "__iadd__",
        ASTAugOpKind::MinusEq => "__isub__",
        ASTAugOpKind::MulEq => "__imul__",
        ASTAugOpKind::DivEq => "__itruediv__",
    }
}

pub fn aug_op_attr_fallbacks(op: ASTAugOpKind) -> ASTBinOpKind {
    match op {
        ASTAugOpKind::PlusEq => ASTBinOpKind::Plus,
        ASTAugOpKind::MinusEq => ASTBinOpKind::Minus,
        ASTAugOpKind::MulEq => ASTBinOpKind::Mul,
        ASTAugOpKind::DivEq => ASTBinOpKind::Div,
    }
}
