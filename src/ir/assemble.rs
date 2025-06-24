use crate::*;

pub fn ir_assemble(toks: &[Token]) -> AST {
    let (ast, toks) = assemble_ast(toks).unwrap();
    assert!(toks.is_empty(), "{:?}", toks);

    ast
}

fn assemble_ast(mut toks: &[Token]) -> Result<(AST, &[Token]), String> {
    let mut stmts = Vec::new();
    while let Ok((x, rst)) = assemble_stmt(toks) {
        toks = rst;
        stmts.push(x);
    }
    Ok((stmts, toks))
}

// (t1, t2, t3, ...)
fn assemble_paren_list<T>(sub: impl Assembler<T>) -> impl Assembler<Vec<T>> {
    assemble_general_list(sub, Token::LParen, Token::RParen)
}

// `l` t1, t2, t3, ... `r`
fn assemble_general_list<T>(sub: impl Assembler<T>, l: Token, r: Token) -> impl Assembler<Vec<T>> {
    move |toks| {
        if toks[0] != l {
            return Err(String::new());
        }
        if toks[1] == r {
            return Ok((Vec::new(), &toks[2..]));
        }
        let toks = &toks[1..];

        let (first, mut toks) = sub(toks)?;
        let mut children = vec![first];

        while toks[0] == Token::Comma {
            toks = &toks[1..];
            let (c, rst) = sub(toks)?;
            toks = rst;
            children.push(c);
        }
        if toks[0] != r {
            return Err(String::new());
        };
        let toks = &toks[1..];
        Ok((children, toks))
    }
}

fn assemble_expr(toks: &[Token]) -> Result<(ASTExpr, &[Token]), String> {
    let (mut expr, mut toks) = assemble_atomic_expr(toks)?;
    loop {
        match toks.get(0) {
            Some(Token::LParen) => {
                let (children, toks2) = assemble_paren_list(assemble_expr)(toks)?;
                expr = ASTExpr::FnCall(Box::new(expr), children);
                toks = toks2;
            }
            Some(Token::BinOp(op)) => {
                let (rhs, toks2) = assemble_expr(&toks[1..])?;
                expr = ASTExpr::BinOp(*op, Box::new(expr), Box::new(rhs));
                toks = toks2;
            }
            Some(Token::Dot) => {
                let (rhs, toks2) = assemble_ident(&toks[1..])?;
                expr = ASTExpr::Attribute(Box::new(expr), rhs);
                toks = toks2;
            },
            Some(Token::LBracket) => {
                let (idx, toks2) = assemble_expr(&toks[1..])?;
                let [Token::RBracket, ..] = &toks2[..] else { return Err("missing RBracket".to_string()) };
                expr = ASTExpr::Index(Box::new(expr), Box::new(idx));
                toks = &toks2[1..];
            },
            _ => return Ok((expr, toks)),
        }
    }
}

fn assemble_atomic_expr(toks: &[Token]) -> Result<(ASTExpr, &[Token]), String> {
    match toks.get(0) {
        Some(Token::Ident(x)) => Ok((ASTExpr::Var(x.clone()), &toks[1..])),
        Some(Token::Int(x)) => Ok((ASTExpr::Int(*x), &toks[1..])),
        Some(Token::Str(s)) => Ok((ASTExpr::Str(s.to_string()), &toks[1..])),
        Some(Token::Bool(b)) => Ok((ASTExpr::Bool(*b), &toks[1..])),
        Some(Token::None) => Ok((ASTExpr::None, &toks[1..])),
        Some(Token::LBracket) => {
            let (list, toks) = assemble_general_list(assemble_expr, Token::LBracket, Token::RBracket)(toks)?;
            Ok((ASTExpr::List(list), toks))
        },
        _ => Err(String::new()),
    }
}

fn assemble_stmt(toks: &[Token]) -> Result<(ASTStatement, &[Token]), String> {
    let a = assemble_stmt_base;
    let a = or(a, assemble_def_stmt);
    let a = or(a, assemble_class_stmt);
    let a = or(a, assemble_expr_stmt);
    let a = or(a, assemble_branch_stmt);
    let a = or(a, assemble_noarg_branch_stmt);
    let a = or(a, assemble_raise_stmt);
    a(toks)
}

fn assemble_raise_stmt(toks: &[Token]) -> Result<(ASTStatement, &[Token]), String> {
    let Some(Token::Raise) = toks.get(0) else { return Err("".to_string()) };
    let toks = &toks[1..];
    let (expr, toks) = assemble_expr(toks)?;
    Ok((ASTStatement::Raise(expr), toks))
}

fn assemble_noarg_branch_stmt(toks: &[Token]) -> Result<(ASTStatement, &[Token]), String> {
    let f = match toks.get(0) {
        Some(Token::Try) => ASTStatement::Try,
        Some(Token::Except) => ASTStatement::Except,
        _ => return Err(String::new()),
    };
    let toks = &toks[1..];
    let (body, toks) = assemble_indented_ast(toks)?;
    Ok((f(body), toks))
}

fn assemble_branch_stmt(toks: &[Token]) -> Result<(ASTStatement, &[Token]), String> {
    let f = match toks.get(0) {
        Some(Token::If) => ASTStatement::If,
        Some(Token::While) => ASTStatement::While,
        _ => return Err(String::new()),
    };
    let toks = &toks[1..];
    let (expr, toks) = assemble_expr(toks)?;
    let (body, toks) = assemble_indented_ast(toks)?;
    Ok((f(expr, body), toks))
}

fn assemble_stmt_base(toks: &[Token]) -> Result<(ASTStatement, &[Token]), String> {
    match toks.get(0) {
        Some(Token::Break) => Ok((ASTStatement::Break, &toks[1..])),
        Some(Token::Continue) => Ok((ASTStatement::Continue, &toks[1..])),
        Some(Token::Pass) => Ok((ASTStatement::Pass, &toks[1..])),
        Some(Token::Scope(kind)) => {
            let (id, toks) = assemble_ident(&toks[1..])?;
            Ok((ASTStatement::Scope(*kind, vec![id]), toks))
        }
        Some(Token::Return) => match assemble_expr(&toks[1..]) {
            Ok((expr, toks)) => Ok((ASTStatement::Return(Some(expr)), toks)),
            Err(_) => Ok((ASTStatement::Return(None), &toks[1..])),
        },
        _ => Err(String::new()),
    }
}

fn assemble_ident(toks: &[Token]) -> Result<(String, &[Token]), String> {
    let Some(Token::Ident(ident_name)) = toks.get(0) else {
        return Err(String::new());
    };
    let toks = &toks[1..];
    Ok((ident_name.to_string(), toks))
}

fn assemble_token(t: Token) -> impl Assembler<()> {
    let opt = Some(t);
    move |toks| {
        if opt.as_ref() == toks.get(0) {
            Ok(((), &toks[1..]))
        } else {
            Err(String::new())
        }
    }
}

fn assemble_def_stmt(toks: &[Token]) -> Result<(ASTStatement, &[Token]), String> {
    let ((), toks) = assemble_token(Token::Def)(toks)?;
    let (fn_name, toks) = assemble_ident(toks)?;
    let (children, toks) = assemble_paren_list(assemble_ident)(toks)?;
    let (body, toks) = assemble_indented_ast(toks)?;
    Ok((ASTStatement::Def(fn_name, children, body), toks))
}

fn assemble_class_stmt(toks: &[Token]) -> Result<(ASTStatement, &[Token]), String> {
    let ((), toks) = assemble_token(Token::Class)(toks)?;
    let (class_name, toks) = assemble_ident(toks)?;
    let (children, toks) = if let Some(Token::LParen) = toks.get(0) {
        assemble_paren_list(assemble_expr)(toks)?
    } else {
        (Vec::new(), toks)
    };
    let (body, toks) = assemble_indented_ast(toks)?;
    Ok((ASTStatement::Class(class_name, children, body), toks))
}

fn assemble_expr_stmt(toks: &[Token]) -> Result<(ASTStatement, &[Token]), String> {
    let (expr, toks) = assemble_expr(toks)?;
    if let Some(Token::Equals) = toks.get(0) {
        let (rhs, toks) = assemble_expr(&toks[1..])?;
        Ok((ASTStatement::Assign(expr, rhs), toks))
    } else {
        Ok((ASTStatement::Expr(expr), toks))
    }
}

fn assemble_indented_ast(toks: &[Token]) -> Result<(AST, &[Token]), String> {
    let ((), toks) = assemble_token(Token::Colon)(toks)?;
    let ((), toks) = assemble_token(Token::Indent)(toks)?;
    let (body, toks) = assemble_ast(toks)?;
    let ((), toks) = assemble_token(Token::Unindent)(toks)?;

    Ok((body, toks))
}

trait Assembler<T>: for<'a> Fn(&[Token]) -> Result<(T, &[Token]), String> {}
impl<A, T> Assembler<T> for A where A: for<'a> Fn(&[Token]) -> Result<(T, &[Token]), String> {}

fn or<T>(a: impl Assembler<T>, b: impl Assembler<T>) -> impl Assembler<T> {
    move |toks| a(toks).or_else(|err| b(toks).map_err(|err2| format!("({err})|({err2})")))
}

fn chain<T1, T2, O>(
    a: impl Assembler<T1>,
    b: impl Assembler<T2>,
    f: impl Fn(T1, T2) -> O,
) -> impl Assembler<O> {
    move |toks| a(toks).and_then(|(x, toks)| b(toks).map(|(y, toks)| (f(x, y), toks)))
}
