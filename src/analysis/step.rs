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

fn step_expr(st: ThreadState, expr: &Expr) -> Vec<(ValueId, ThreadState)> {
    match expr {
        Expr::Index(_, _) => todo!(),
        Expr::Root => todo!(),
        Expr::NewTable => todo!(),
        Expr::BinOp(_, _, _) => todo!(),
        Expr::Input => todo!(),

        Expr::Symbol(_) => todo!(),
        Expr::Float(_) => todo!(),
        Expr::Int(_) => todo!(),
        Expr::Str(_) => todo!(),
    }
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
