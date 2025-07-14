use crate::*;

impl AnalysisState {
    pub fn step(&mut self, i: SpecId) {
        let states = {
            let spec = &self.specs[&i];
            let mut states = vec![spec.st.clone()];
            for stmt in self.ir.procs[&spec.st.pid].stmts.iter() {
                let mut new_states = Vec::new();
                for st in states {
                    new_states.extend(step_stmt(st, stmt));
                }
                states = new_states;
            }
            states
        };

        for st in states {
            let id: SpecId = Id(self.specs.len());
            self.queue.push_back(id);

            let spec = Spec {
                st,
                outs: Default::default(),
            };
            self.specs.insert(id, spec);
        }
    }
}

fn step_stmt(st: ThreadState, stmt: &Statement) -> Vec<ThreadState> {
    todo!()
}
