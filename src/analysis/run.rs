use crate::*;

// returns true for safe, and false for unsafe.
pub fn analyze(ir: IR) -> bool {
    let analysis = build_analysis(ir);
    check_analysis_safe(&analysis)
}

fn build_analysis(ir: IR) -> AnalysisState {
    let mut analysis = AnalysisState {
        ir,
        specs: Map::new(),
        queue: Default::default(),
    };

    let spec_id = SpecId(Symbol::new_fresh("startSpecId"));
    analysis.queue.push(spec_id);

    let spec = {
        let root_id = ValueId(Symbol::new_fresh("rootValueId"));
        let root_sort_id = TableSortId(Symbol::new_fresh("rootTableSortId"));

        let mut deref: Map<_, _> = Default::default();
        let vs = ValueSet(vec![ValueParticle::TableSort(root_sort_id)]);
        deref.insert(root_id, vs);

        Spec {
            st: ThreadState {
                tkvs: Default::default(),
                ts_cache: Default::default(),
                deref,
                root: root_id,
                pid: analysis.ir.main_pid,
                nodes: Map::new(),
            },
            outs: Vec::new(),
        }
    };

    analysis.specs.insert(spec_id, spec);

    while let Some(i) = analysis.queue.pop() {
        analysis.step(i);
    }

    analysis
}

fn check_analysis_safe(analysis: &AnalysisState) -> bool {
    for (_, b) in &analysis.specs {
        for stmt in analysis.ir.procs[&b.st.pid].stmts.iter() {
            if matches!(stmt, Statement::Fail) { return false; }
        }
    }
    true
}

