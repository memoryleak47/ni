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

const HIST_LEN: usize = 5;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Hist(Vec<Symbol>);

#[derive(Debug)]
pub struct AnalysisState {
    pub states: Map<Hist, ProcState>,
    pub queue: Vec<Hist>,
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
    pub fn add(&mut self, hist: Hist, st: ProcState) {
        match self.states.entry(hist.clone()) {
            indexmap::map::Entry::Vacant(e) => {
                e.insert(st);
                if !self.queue.contains(&hist) { self.queue.push(hist); }
            }
            indexmap::map::Entry::Occupied(mut e) => {
                let s = e.get_mut();
                let changed = s.merge(&st);
                if changed && !self.queue.contains(&hist) { self.queue.push(hist); }
            },
        }
    }
}

impl Hist {
    pub fn step(&mut self, next: Symbol) {
        self.0.retain(|x| *x != next);
        if self.0.len() == HIST_LEN {
            self.0.remove(0);
        }
        self.0.push(next);
    }

    pub fn head(&self) -> Symbol {
        *self.0.last().unwrap()
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
