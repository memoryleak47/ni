use crate::*;

pub fn ir_assemble(toks: &[IRToken]) -> IR {
    let (ir, toks) = assemble_ir(toks).unwrap();
    assert!(toks.is_empty(), "{:?}", toks);

    ir
}

fn assemble_ir(mut toks: &[IRToken]) -> Option<(IR, &[IRToken])> {
    let mut procs = Map::new();
    let mut main_pid = None;
    while let Some((start, pid, x, rst)) = assemble_proc(toks) {
        toks = rst;
        if start {
            assert!(main_pid.is_none());
            main_pid = Some(pid);
        }
        procs.insert(pid, x);
    }
    let ir = IR {
        procs,
        main_pid: main_pid.unwrap_or_else(|| ProcId(gsymb_add("todo".to_string()))),
    };
    Some((ir, toks))
}

fn assemble_proc(mut toks: &[IRToken]) -> Option<(/*start*/ bool, ProcId, Procedure, &[IRToken])> {
    let mut main = false;
    if let [IRToken::Main, ..] = toks {
        main = true;
        toks = &toks[1..];
    }
    let [IRToken::Proc, IRToken::Symbol(pid), IRToken::LBrace, toks@..] = toks else { return None };
    let pid = ProcId(*pid);

    let mut toks = toks;
    let mut stmts = Vec::new();
    while let Some((x, toks2)) = assemble_stmt(toks) {
        stmts.push(x);
        toks = toks2;
    }
    let (terminator, toks) = assemble_terminator(toks)?;

    let [IRToken::RBrace, toks@..] = toks else { return None };

    let proc = Procedure {
        stmts,
        terminator
    };
    Some((main, pid, proc, toks))
}

fn assemble_stmt(toks: &[IRToken]) -> Option<(Statement, &[IRToken])> {
    let a = assemble_stmt_let;
    let a = or(a, assemble_stmt_store);
    let a = or(a, assemble_stmt_print);
    a(toks)
}

fn assemble_stmt_let(toks: &[IRToken]) -> Option<(Statement, &[IRToken])> { None }
fn assemble_stmt_store(toks: &[IRToken]) -> Option<(Statement, &[IRToken])> { None }
fn assemble_stmt_print(toks: &[IRToken]) -> Option<(Statement, &[IRToken])> { None }

fn assemble_terminator(toks: &[IRToken]) -> Option<(Terminator, &[IRToken])> {
    or(assemble_terminator_jmp, assemble_terminator_exit)(toks)
}

fn assemble_terminator_jmp(toks: &[IRToken]) -> Option<(Terminator, &[IRToken])> { None }
fn assemble_terminator_exit(toks: &[IRToken]) -> Option<(Terminator, &[IRToken])> { None }

fn assemble_expr(toks: &[IRToken]) -> Option<(Statement, &[IRToken])> {
    None
}

trait Assembler<T>: for<'a> Fn(&[IRToken]) -> Option<(T, &[IRToken])> {}
impl<A, T> Assembler<T> for A where A: for<'a> Fn(&[IRToken]) -> Option<(T, &[IRToken])> {}

fn or<T>(a: impl Assembler<T>, b: impl Assembler<T>) -> impl Assembler<T> {
    move |toks| a(toks).or_else(|| b(toks))
}
