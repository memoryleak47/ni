use crate::standard_analysis::*;

impl AnalysisState {
    pub fn step(&mut self, pid: Symbol) {
        let state = self.states.get(&pid).unwrap_or_else(|| panic!("Spec '{pid:?}' not found!"));
        let mut states: Vec<ProcState> = vec![state.clone()];
        for (i, stmt) in self.ir.procs[&pid].stmts.iter().enumerate() {
            let mut new_states = Vec::new();
            for st in states {
                let loc = (pid, i);
                new_states.extend(step_stmt(st, stmt, loc, &self.ir));
            }
            states = new_states;
        }

        for st in states {
            self.add(st);
        }
    }
}

fn step_expr(mut st: ProcState, loc: Location, expr: &Expr) -> (ValueSet, ProcState) {
    match expr {
        Expr::Index(t, k) => {
            let t = &st.nodes[t];
            let k = &st.nodes[k];
            let vs = tab_index(t, k, &st);
            (vs, st)
        },
        Expr::Root => (ValueSet(vec![st.root.clone()]), st),
        Expr::NewTable => {
            st.summarize(loc);
            let vs = ValueSet(vec![ValueParticle::Concrete(loc)]);
            (vs, st)
        },
        Expr::BinOp(kind, l, r) => {
            let l = st.nodes[l].clone();
            let r = st.nodes[r].clone();
            step_binop(*kind, l, r, st)
        },
        Expr::Input => {
            let vs = ValueSet(vec![ValueParticle::TopString]);
            (vs, st)
        },

        Expr::Symbol(s) => (ValueSet(vec![ValueParticle::Symbol(*s)]), st),
        Expr::Float(_) => todo!(),
        Expr::Int(i) => (ValueSet(vec![ValueParticle::Int(*i)]), st),
        Expr::Str(s) => (ValueSet(vec![ValueParticle::String(s.clone())]), st),
    }
}

fn step_stmt(mut st: ProcState, stmt: &Statement, loc: Location, ir: &IR) -> Vec<ProcState> {
    match stmt {
        Statement::Let(n, expr, _) => {
            let (vs, mut new_st) = step_expr(st, loc, expr);
            new_st.nodes.insert(*n, vs);
            vec![new_st]
        }
        Statement::Store(t, k, v) => {
            let t = st.nodes[t].clone();
            let k = st.nodes[k].clone();
            let v = st.nodes[v].clone();
            tab_store(&t, &k, &v, &mut st);
            vec![st]
        },
        Statement::Jmp(n) => {
            let vs = st.nodes[n].clone();

            st.nodes.clear();

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
