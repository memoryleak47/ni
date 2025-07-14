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
            let tovs = |x| {
                let mut out = ValueSet::bottom();
                out.value_ids.insert(st.nodes[x]);
                out
            };

            let t = tovs(t);
            let k = tovs(k);

            let mut out = ValueSet::top();

            for (t2, k2, v2) in st.tkvs.iter() {
                if t.is_subset(&t2, &st) && k.is_subset(&k2, &st) {
                    vs = vs.intersection(&v2, &st);
                }
            }
        },
        Expr::Root => return vec![(st.root, st)],
        Expr::NewTable => {
            let sort_id = Symbol::new_fresh("sortId");

            { // add (value_id, Top, Undef) triple!
                let mut t = ValueSet::bottom();
                t.value_ids.insert(value_id);

                let mut undef = ValueSet::bottom();
                undef.symbols.insert(Symbol::new("Undef"));

                st.tkvs.push((t, ValueSet::top(), undef));
            }

            vs.table_sorts.insert(sort_id);
        },
        Expr::BinOp(_, _, _) => todo!(),
        Expr::Input => { vs.strings = OrTop::Top; },

        Expr::Symbol(s) => { vs.symbols.insert(*s); },
        Expr::Float(_) => todo!(),
        Expr::Int(i) => vs.ints.insert(*i),
        Expr::Str(s) => vs.strings.insert(s.clone()),
    };

    st.deref.insert(value_id, vs);

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
        Statement::Store(t, k, v) => {
            let tovs = |x| {
                let mut out = ValueSet::bottom();
                out.value_ids.insert(st.nodes[x]);
                out
            };

            let t = tovs(t);
            let k = tovs(k);
            let v = tovs(v);

            // remove strictly overwritten stuff.
            st.tkvs.retain(|(t2, k2, _)| !t.concrete_eq(t2) || !k.concrete_eq(k2));

            for (t2, k2, v2) in st.tkvs.iter_mut() {
                if t.overlaps(&*t2, &st.deref) && k.overlaps(&*k2, &st.deref) {
                    *v2 = v2.union(&v);
                }
            }

            st.tkvs.push((t, k, v));

            vec![st]
        }
        Statement::Jmp(n) => {
            let vid = st.nodes[n];
            let vs = full_deref(vid, &st.deref);

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
