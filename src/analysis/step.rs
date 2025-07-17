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
            let id = SpecId(Symbol::new_fresh("specId"));
            self.queue.push(id);

            let spec = Spec {
                st,
                outs: Default::default(),
            };
            self.specs.insert(id, spec);
        }
    }
}

fn step_expr(mut st: ThreadState, expr: &Expr) -> (ValueId, ThreadState) {
    let mut value_id = ValueId(Symbol::new_fresh("valueId"));
    let mut vs = ValueSet::bottom();
    match expr {
        Expr::Index(t, k) => {
            let t = &st.nodes[t];
            let k = &st.nodes[k];

            vs = index_p(t, k, &st, &mut Default::default());
        },
        Expr::Root => return (st.root, st),
        Expr::NewTable => {
            let sort_id = TableSortId(Symbol::new_fresh("sortId"));

            vs = ValueSet(vec![ValueParticle::TableSort(sort_id)]);
        },
        Expr::BinOp(_, _, _) => todo!(),
        Expr::Input => { vs = ValueSet(vec![ValueParticle::TopString]); },

        Expr::Symbol(s) => { vs = ValueSet(vec![ValueParticle::Symbol(*s)]); },
        Expr::Float(_) => todo!(),
        Expr::Int(i) => { vs = ValueSet(vec![ValueParticle::Int(*i)]); },
        Expr::Str(s) => { vs = ValueSet(vec![ValueParticle::String(s.clone())]); },
    };

    st.deref.insert(value_id, vs);

    (value_id, st)
}

fn step_stmt(mut st: ThreadState, stmt: &Statement) -> Vec<ThreadState> {
    match stmt {
        Statement::Let(n, expr, _) => {
            let (val_id, mut new_st) = step_expr(st, expr);
            new_st.nodes.insert(*n, ValueParticle::ValueId(val_id));
            vec![new_st]
        }
        Statement::Store(t, k, v) => {
            let t = st.nodes[t].clone();
            let k = st.nodes[k].clone();
            let v = ValueSet(vec![st.nodes[v].clone()]);
            let st = store_p(&t, &k, v, st);
            vec![st]
        },
        Statement::Jmp(n) => {
            let vid = st.nodes[n].clone();
            let vs = vid.deref(&st.deref);

            st.nodes.clear();
            let mut outs = Vec::new();
            let procs = vs.0.iter().filter_map(|x| match x {
                ValueParticle::Symbol(s) => Some(*s),
                _ => None,
            });
            for pid in procs {
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
