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
            let id: SpecId = Symbol::new_fresh("specId");
            self.queue.push_back(id);

            let spec = Spec {
                st,
                outs: Default::default(),
            };
            self.specs.insert(id, spec);
        }
    }
}

fn step_expr(mut st: ThreadState, expr: &Expr) -> Vec<(ValueId, ThreadState)> {
    let mut value_id = Symbol::new_fresh("valueId");
    let mut vs = ValueSet::bottom();
    match expr {
        Expr::Index(t, k) => {
            todo!()
        },
        Expr::Root => return vec![(st.root, st)],
        Expr::NewTable => {
            let sort_id = Symbol::new_fresh("sortId");
            vs.table_sorts.insert(sort_id);
        },
        Expr::BinOp(_, _, _) => todo!(),
        Expr::Input => todo!(),

        Expr::Symbol(s) => { vs.symbols.insert(*s); },
        Expr::Float(_) => todo!(),
        Expr::Int(i) => vs.ints.insert(*i),
        Expr::Str(s) => vs.strings.insert(s.clone()),
    };

    st.deref_val_id.insert(value_id, vs);

    vec![(value_id, st)]
}

fn step_stmt(mut st: ThreadState, stmt: &Statement) -> Vec<ThreadState> {
    match stmt {
        Statement::Let(n, expr, _) => {
            let mut out = Vec::new();
            for (val_id, mut x) in step_expr(st, expr) {
                x.nodes.insert(*n, val_id);
                out.push(x);
            }
            out
        }
        Statement::Store(t, idx, n) => {
            let tovs = |x| {
                let mut out = ValueSet::bottom();
                out.value_ids.insert(st.nodes[x]);
                out
            };

            let t = tovs(t);
            let idx = tovs(idx);
            let n = tovs(n);

            st.tkvs.push((t, idx, n));
            vec![st]
        }
        Statement::Jmp(n) => {
            let vid = st.nodes[n];
            let vs = full_deref(vid, &st);

            st.nodes.clear();
            let mut outs = Vec::new();
            for pid in vs.symbols {
                let mut st = st.clone();
                st.pid = pid;
                outs.push(st);
            }
            outs
        }
        Statement::Print(_) => vec![st],
        Statement::Exit | Statement::Panic(_) | Statement::Fail => Vec::new(),
    }
}
