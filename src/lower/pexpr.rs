use crate::lower::*;

pub enum PlaceExpr {
    Subscript(Lowered, Lowered), // lhs[i]
    Attr(Lowered, String), // lhs.attr
    Var(String), // v
}

pub fn lower_pexpr(e: &ASTExpr, ctxt: &mut Ctxt) -> PlaceExpr {
    match e {
        ASTExpr::Var(v) => PlaceExpr::Var(v.clone()),
        ASTExpr::Attribute(e, v) => {
            let e = lower_expr(e, ctxt);
            PlaceExpr::Attr(e, v.to_string())
        },
        ASTExpr::BinOp(ASTBinOpKind::Subscript, e, i) => {
            let e = lower_expr(e, ctxt);
            let i = lower_expr(i, ctxt);
            PlaceExpr::Subscript(e, i)
        },
        _ => panic!("Not a PlaceExpr: {e:?}"),
    }
}

pub fn pexpr_load(e: &PlaceExpr, ctxt: &mut Ctxt) -> Lowered {
    match e {
        PlaceExpr::Var(var) => {
            let ns = find_namespace(var, ctxt);
            format!("{ns}[\"{var}\"]")
        },
        PlaceExpr::Attr(e, a) => {
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
        PlaceExpr::Subscript(e, i) => todo!(),
    }
}

pub fn pexpr_store(e: &PlaceExpr, val: Lowered, ctxt: &mut Ctxt) {
    match e {
        PlaceExpr::Var(var) => {
            lower_var_assign(var, val, ctxt);
        },
        PlaceExpr::Attr(e, v) => {
            ctxt.push(format!("{e}.dict[\"{v}\"] = {val}"));
        },
        PlaceExpr::Subscript(e, i) => {
            todo!()
/*
            let e_setattr = Box::new(ASTExpr::Attribute(e.clone(), "__setitem__".to_string()));
            let real_stmt = ASTStatement::Expr(ASTExpr::FnCall(e_setattr, vec![(**v).clone(), rhs.clone()]));
            lower_body(&[real_stmt], ctxt);
*/
        },
    }
}


