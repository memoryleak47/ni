use crate::*;

mod valueset;
pub use valueset::*;

mod run;
pub use run::*;

mod step;

mod binop;
pub use binop::*;

mod table;
pub use table::*;

mod fmt;

#[derive(Debug)]
pub struct AnalysisState {
    pub states: Map<Symbol, ProcState>,
    pub queue: Vec<Symbol>,
    pub ir: IR,
}

#[derive(Clone, Debug)]
pub struct ProcState {
    // any missing entry is Undef.
    pub tables: Map<[ValueParticle; 2], ValueSet>,

    pub root: ValueParticle,
    pub pid: Symbol,

    pub nodes: Map<Node, ValueSet>,
}

impl AnalysisState {
    pub fn add(&mut self, st: ProcState) {
        let pid = st.pid;
        match self.states.entry(pid) {
            indexmap::map::Entry::Vacant(e) => {
                e.insert(st);
                if !self.queue.contains(&pid) { self.queue.push(pid); }
            }
            indexmap::map::Entry::Occupied(mut e) => {
                let s = e.get_mut();
                let changed = s.merge(&st);
                if changed && !self.queue.contains(&pid) { self.queue.push(pid); }
            },
        }
    }
}

impl ProcState {
    pub fn summarize(&mut self, loc: Location) {
        assert!(self.root != ValueParticle::Concrete(loc), "Why would you summarize root?");

        for (_, vs) in self.nodes.iter_mut() {
            summarize_set(loc, vs);
        }

        for ([mut t, mut k], mut v) in std::mem::take(&mut self.tables) {
            summarize_set(loc, &mut v);
            if t == ValueParticle::Concrete(loc) { t = ValueParticle::Summary(loc); }
            if k == ValueParticle::Concrete(loc) { k = ValueParticle::Summary(loc); }
            let vv = self.tables.entry([t, k]).or_default();
            *vv = vv.union(&v);
        }
    }

    // returns whether "self" was changed.
    pub fn merge(&mut self, other: &ProcState) -> bool {
        assert!(self.nodes.is_empty());
        assert!(other.nodes.is_empty());
        assert_eq!(self.root, other.root);
        assert_eq!(self.pid, other.pid);

        let mut changed = false;
        for ([t, k], v) in &other.tables {
            let tk = [t.clone(), k.clone()];
            let my_v = self.tables.entry(tk).or_default();
            if !v.subseteq(my_v) {
                *my_v = my_v.union(v);
                changed = true;
            }
        }
        changed
    }
}

fn summarize_set(loc: Location, vs: &mut ValueSet) {
    let n = vs.0.len();
    vs.0.retain(|x| *x != ValueParticle::Concrete(loc));
    if n != vs.0.len() && !vs.0.contains(&ValueParticle::Summary(loc)) {
        vs.0.push(ValueParticle::Summary(loc));
    }
}
