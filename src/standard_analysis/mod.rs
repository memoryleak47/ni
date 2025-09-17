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
        match self.states.entry(st.pid) {
            indexmap::map::Entry::Vacant(e) => { e.insert(st); },
            indexmap::map::Entry::Occupied(mut e) => {
                e.insert(e.get().union(&st));
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

    // TODO: also return whether there actually was something new.
    pub fn union(&self, other: &ProcState) -> ProcState {
        todo!()
    }
}

fn summarize_set(loc: Location, vs: &mut ValueSet) {
    let n = vs.0.len();
    vs.0.retain(|x| *x != ValueParticle::Concrete(loc));
    if n != vs.0.len() && !vs.0.contains(&ValueParticle::Summary(loc)) {
        vs.0.push(ValueParticle::Summary(loc));
    }
}
