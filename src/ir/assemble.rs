use crate::*;

pub fn ir_assemble(mut toks: &[IRToken]) -> IR {
    let mut procs = Map::new();
    let mut main_pid = None;
    while toks.len() > 0 {
        let (start, pid, x, rst) = assemble_proc(toks);
        toks = rst;
        if start {
            assert!(main_pid.is_none());
            main_pid = Some(pid);
        }
        procs.insert(pid, x);
    }
    IR {
        procs,
        main_pid: main_pid.unwrap(),
    }
}

fn assemble_proc(mut toks: &[IRToken]) -> (/*start*/ bool, ProcId, Proc, &[IRToken]) {
    let mut main = false;
    if let [IRToken::Main, ..] = toks {
        main = true;
        toks = &toks[1..];
    }
    let [IRToken::Proc, IRToken::Symbol(pid), IRToken::LBrace, toks@..] = toks else { panic!() };
    let pid = ProcId(*pid);

    let mut toks = toks;
    let mut stmts = Vec::new();
    while let Some((x, prev, toks2)) = assemble_stmt(toks) {
        stmts.extend(prev);
        stmts.push(x);
        toks = toks2;
    }
    let (terminator, prev, toks) = assemble_terminator(toks).unwrap_or_else(|| {
        panic!("Couldn't parse stmt or terminator starting at:\n{toks:?}")
    });
    stmts.extend(prev);

    let [IRToken::RBrace, toks@..] = toks else { panic!("missing }}!") };

    let proc = Proc {
        stmts,
        terminator
    };
    (main, pid, proc, toks)
}

fn assemble_stmt(toks: &[IRToken]) -> Option<(Statement, Vec<Statement>, &[IRToken])> {
    let a = assemble_stmt_let;
    let a = or(a, assemble_stmt_store);
    let a = or(a, assemble_stmt_print);
    let (stmt, prev, toks) = a(toks)?;
    let [IRToken::Semicolon, toks@..] = toks else { return None };
    Some((stmt, prev, toks))
}

fn assemble_stmt_let(toks: &[IRToken]) -> Option<(Statement, Vec<Statement>, &[IRToken])> {
    let [IRToken::BinOp(BinOpKind::Mod), IRToken::Symbol(node), IRToken::Equals, toks@..] = &toks[..] else { return None };
    let node = Node(*node);
    let (expr, prev, toks) = assemble_expr(toks)?;
    Some((Statement::Let(node, expr, true), prev, toks))
}

fn assemble_stmt_store(toks: &[IRToken]) -> Option<(Statement, Vec<Statement>, &[IRToken])> {
    let (expr, mut prev, toks) = assemble_expr(toks)?;
    let [IRToken::Equals, toks@..] = toks else { return None };
    let (rhs, prev2, toks) = assemble_expr_node(toks)?;
    prev.extend(prev2);

    let Expr::Index(tab, idx) = expr else { return None };
    Some((Statement::Store(tab, idx, rhs), prev, toks))
}

fn assemble_stmt_print(toks: &[IRToken]) -> Option<(Statement, Vec<Statement>, &[IRToken])> {
    let [IRToken::Print, toks@..] = toks else { return None };
    let (node, prev, toks) = assemble_expr_node(toks)?;
    Some((Statement::Print(node), prev, toks))
}

fn assemble_terminator(toks: &[IRToken]) -> Option<(Terminator, Vec<Statement>, &[IRToken])> {
    let a = assemble_terminator_jmp;
    let a = or(a, assemble_terminator_exit);
    let a = or(a, assemble_terminator_panic);

    let (terminator, prev, toks) = a(toks)?;
    let [IRToken::Semicolon, toks@..] = toks else { return None };
    Some((terminator, prev, toks))
}

fn assemble_terminator_jmp(toks: &[IRToken]) -> Option<(Terminator, Vec<Statement>, &[IRToken])> {
    let [IRToken::Jmp, toks@..] = toks else { return None };
    let (node, prev, toks) = assemble_expr_node(toks)?;
    Some((Terminator::Jmp(node), prev, toks))
}

fn assemble_terminator_exit(toks: &[IRToken]) -> Option<(Terminator, Vec<Statement>, &[IRToken])> {
    let [IRToken::Exit, toks@..] = toks else { return None };
    Some((Terminator::Exit, Vec::new(), toks))
}

fn assemble_terminator_panic(toks: &[IRToken]) -> Option<(Terminator, Vec<Statement>, &[IRToken])> {
    let [IRToken::Panic, toks@..] = toks else { return None };
    let (node, prev, toks) = assemble_expr_node(toks)?;
    Some((Terminator::Panic(node), prev, toks))
}

fn assemble_expr(toks: &[IRToken]) -> Option<(Expr, Vec<Statement>, &[IRToken])> {
    let (mut expr, mut prev, mut toks) = assemble_atomic_expr(toks)?;
    loop {
        match toks {
            [IRToken::Dot, IRToken::Symbol(s), toks2@..] => {
                let expr_node = Node(Symbol::fresh());
                prev.push(Statement::Let(expr_node, expr, false));

                let node = Node(Symbol::fresh());
                let e = Expr::Symbol(*s);
                prev.push(Statement::Let(node, e, false));
                expr = Expr::Index(expr_node, node);
                toks = toks2;
            },
            [IRToken::LBracket, toks2@..] => {
                let expr_node = Node(Symbol::fresh());
                prev.push(Statement::Let(expr_node, expr, false));

                let (node, prev2, toks2) = assemble_expr_node(toks2)?;
                let [IRToken::RBracket, toks2@..] = toks2 else { return None };
                prev.extend(prev2);
                expr = Expr::Index(expr_node, node);
                toks = toks2;
            },
            _ => return Some((expr, prev, toks)),
        }
    }
}

fn assemble_atomic_expr(toks: &[IRToken]) -> Option<(Expr, Vec<Statement>, &[IRToken])> {
    match &toks[..] {
        [IRToken::At, toks@..] => Some((Expr::Root, Vec::new(), toks)),
        [IRToken::Dollar, IRToken::Symbol(s), toks@..] => Some((Expr::Symbol(*s), Vec::new(), toks)),
        [IRToken::Symbol(s), toks@..] => Some((Expr::Proc(ProcId(*s)), Vec::new(), toks)),
        [IRToken::Int(i), toks@..] => Some((Expr::Int(*i), Vec::new(), toks)),
        [IRToken::LBrace, IRToken::RBrace, toks@..] => Some((Expr::NewTable, Vec::new(), toks)),
        [IRToken::LParen, toks@..] => {
            let (expr, prev, toks) = assemble_expr(toks)?;
            let [IRToken::RParen, toks@..] = toks else { return None };
            Some((expr, prev, toks))
        },
        _ => None,
    }
}

fn assemble_expr_node(toks: &[IRToken]) -> Option<(Node, Vec<Statement>, &[IRToken])> {
    if let [IRToken::BinOp(BinOpKind::Mod), IRToken::Symbol(s), toks@..] = toks {
        return Some((Node(*s), Vec::new(), toks));
    }

    let (expr, mut prev, toks) = assemble_expr(toks)?;
    let node = Node(Symbol::fresh());
    let mut let_stmt = Statement::Let(node, expr, false);
    prev.push(let_stmt);
    Some((node, prev, toks))
}

trait Assembler<T>: for<'a> Fn(&[IRToken]) -> Option<(T, Vec<Statement>, &[IRToken])> {}
impl<A, T> Assembler<T> for A where A: for<'a> Fn(&[IRToken]) -> Option<(T, Vec<Statement>, &[IRToken])> {}

fn or<T>(a: impl Assembler<T>, b: impl Assembler<T>) -> impl Assembler<T> {
    move |toks| a(toks).or_else(|| b(toks))
}
