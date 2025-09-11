use crate::*;

// returns true for safe, and false for unsafe.
pub fn analyze(ir: IR) -> bool {
    let analysis = build_analysis(ir);
    check_analysis_safe(&analysis)
}

fn build_analysis(ir: IR) -> AnalysisState {
    let mut analysis = AnalysisState {
        root_spec: SpecId(Symbol::new("missing")), // will be set correctly below.
        ir,
        specs: Map::new(),
        queue: Default::default(),
        heur_queue: Default::default(),
    };

    let st = {
        let root_id = ValueId(Symbol::new_fresh("rootValueId"));
        let root_sort_id = TableSortId(Symbol::new_fresh("rootTableSortId"));
            let mut deref: Map<_, _> = Default::default();
            let vs = ValueSet(vec![ValueParticle::TableSort(root_sort_id)]);
            deref.insert(root_id, vs);

        ThreadState {
            table_entries: Default::default(),
            deref,
            root: root_id,
            pid: analysis.ir.main_pid,
            nodes: Map::new(),
        }
    };
    let spec_id = analysis.add(st);
    analysis.root_spec = spec_id;

    loop {
        while let Some(x) = analysis.heur_queue.pop() {
            heur(&mut analysis, x);
        }

        let Some(i) = analysis.queue.pop() else { break };
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

impl AnalysisState {
    pub fn add(&mut self, st: ThreadState) -> SpecId {
        if CHECKS {
            st.check();
        }

        let spec_id = SpecId(Symbol::new_fresh(&format!("specId_{}", st.pid)));
        let spec = Spec { st, outs: Vec::new() };
        self.specs.insert(spec_id, spec);

        self.heur_queue.push(spec_id);

        spec_id
    }
}
