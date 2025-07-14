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
        Expr::Index(_, _) => todo!(),
        Expr::Root => return vec![(st.root, st)],
        Expr::NewTable => todo!(),
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
            todo!()
        }
        Statement::Jmp(n) => {
            todo!()
        }
        Statement::Print(_) => vec![st],
        Statement::Exit | Statement::Panic(_) | Statement::Fail => Vec::new(),
    }
}
