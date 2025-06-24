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
        main_pid: main_pid.unwrap(),
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
    let proc = Procedure {
        stmts,
        terminator
    };
    Some((main, pid, proc, toks))
}

fn assemble_stmt(toks: &[IRToken]) -> Option<(Statement, &[IRToken])> {
    todo!()
}

fn assemble_terminator(toks: &[IRToken]) -> Option<(Terminator, &[IRToken])> {
    todo!()
}
