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

fn assemble_proc(toks: &[IRToken]) -> Option<(/*start*/ bool, ProcId, Procedure, &[IRToken])> {
    todo!()
}
