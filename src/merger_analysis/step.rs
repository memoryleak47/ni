use crate::merger_analysis::*;

impl AnalysisState {
    pub fn step(&mut self, i: SpecId) {
        let spec = &self.specs.get(&i).unwrap_or_else(|| panic!("Spec '{i:?}' not found!"));
        let mut states = vec![spec.st.clone()];
        for stmt in self.ir.procs[&spec.st.pid].stmts.iter() {
            let mut new_states = Vec::new();
            for st in states {
                new_states.extend(step_stmt(st, stmt, &self.ir));
            }
            states = new_states;
        }

        let mut outs = Vec::new();
        for st in states {
            outs.push(self.add(st));
        }
        self.specs[&i].outs = outs;
    }
}

fn step_expr(mut st: ThreadState, expr: &Expr) -> (ValueParticle, ThreadState) {
    match expr {
        Expr::Index(t, k) => {
            let t = &st.nodes[t];
            let k = &st.nodes[k];

            let vs = index_p(t, k, &st);
            if let [x] = &*vs.0 && x.is_concrete() {
                return (x.clone(), st);
            }

            let value_id = ValueId(Symbol::new_fresh("indexVID"));
            st.deref.insert(value_id, vs);
            (ValueParticle::ValueId(value_id), st)
        },
        Expr::Root => return (ValueParticle::ValueId(st.root), st),
        Expr::NewTable => {
            let value_id = ValueId(Symbol::new_fresh("tableVID"));
            let sort_id = TableSortId(Symbol::new_fresh("sortId"));
            let vs = ValueSet(vec![ValueParticle::TableSort(sort_id)]);
            st.deref.insert(value_id, vs);

            // Note: We could also handle this in `index_p`.
            st = store_p(ValueParticle::ValueId(value_id), ValueParticle::Top, ValueParticle::Symbol(Symbol::new("Undef")), st);

            (ValueParticle::ValueId(value_id), st)
        },
        Expr::BinOp(kind, l, r) => {
            let l = st.nodes[l].clone();
            let r = st.nodes[r].clone();
            step_binop(*kind, l, r, st)
        },
        Expr::Input => {
            let value_id = ValueId(Symbol::new_fresh("inputVID"));
            let vs = ValueSet(vec![ValueParticle::TopString]);
            st.deref.insert(value_id, vs);
            (ValueParticle::ValueId(value_id), st)
        },

        Expr::Symbol(s) => (ValueParticle::Symbol(*s), st),
        Expr::Float(_) => todo!(),
        Expr::Int(i) => (ValueParticle::Int(*i), st),
        Expr::Str(s) => (ValueParticle::String(s.clone()), st),
    }
}

fn step_stmt(mut st: ThreadState, stmt: &Statement, ir: &IR) -> Vec<ThreadState> {
    match stmt {
        Statement::Let(n, expr, _) => {
            let (part, mut new_st) = step_expr(st, expr);
            new_st.nodes.insert(*n, part);
            vec![new_st]
        }
        Statement::Store(t, k, v) => {
            let t = st.nodes[t].clone();
            let k = st.nodes[k].clone();
            let v = st.nodes[v].clone();
            let st = store_p(t, k, v, st);
            vec![st]
        },
        Statement::Jmp(n) => {
            let vid = st.nodes[n].clone();
            let vs = vid.deref(&st.deref);

            st.nodes.clear();
            gc_ts(&mut st);

            let mut outs = Vec::new();
            let procs = vs.0.iter().filter_map(|x| match x {
                ValueParticle::Symbol(s) if ir.procs.contains_key(s) => Some(*s),
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
