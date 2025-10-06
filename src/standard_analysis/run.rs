use crate::standard_analysis::*;

// returns true for safe, and false for unsafe.
pub fn analyze(ir: IR) -> bool {
    let analysis = build_analysis(ir);
    check_analysis_safe(&analysis)
}

fn build_analysis(ir: IR) -> AnalysisState {
    let main_pid = ir.main_pid;
    let main_hist = Hist(vec![main_pid]);
    let mut analysis = AnalysisState {
        ir,
        states: Map::new(),
        queue: vec![main_hist.clone()],
    };

    let st = ProcState {
        tables: Default::default(),
        root: ValueParticle::Concrete((Symbol::new("$START"), 0)),
        pid: main_pid,
        nodes: Map::new(),
    };
    analysis.states.insert(main_hist, st);

    while let Some(s) = analysis.queue.pop() {
        analysis.step(s);
    }

    analysis
}

fn check_analysis_safe(analysis: &AnalysisState) -> bool {
    for (s, _) in &analysis.states {
        let pid = s.head();
        for stmt in analysis.ir.procs[&pid].stmts.iter() {
            if matches!(stmt, Statement::Fail) { return false; }
        }
    }
    true
}
