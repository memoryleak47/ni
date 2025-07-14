use crate::*;

impl AnalysisState {
    pub fn step(&mut self, i: SpecId) {
        let states = {
            let spec = &self.specs[&i];
            let fst = FullThreadState {
                st: spec.st.clone(),
                nodes: Map::new(),
            };
            let mut states = vec![fst];
            for stmt in self.ir.procs[&spec.st.pid].stmts.iter() {
                let mut new_states = Vec::new();
                for st in states {
                    new_states.extend(step_stmt(st, stmt));
                }
                states = new_states;
            }
            states
        };

        for fst in states {
            let id: SpecId = Id(self.specs.len());
            self.queue.push_back(id);

            let spec = Spec {
                st: fst.st,
                outs: Default::default(),
            };
            self.specs.insert(id, spec);
        }
    }
}

struct FullThreadState {
    st: ThreadState,
    nodes: Map<Node, ValueId>,
}

fn step_expr(st: FullThreadState, expr: &Expr) -> Vec<(ValueId, FullThreadState)> {
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

fn step_stmt(st: FullThreadState, stmt: &Statement) -> Vec<FullThreadState> {
    match stmt {
        Statement::Let(n, expr, _) => {
            // let val = step_expr(expr, ctxt);
            // ctxt.nodes.insert(*n, val);
            todo!()
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
