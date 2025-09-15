use crate::*;

mod valueset;
pub use valueset::*;

mod table;
pub use table::*;

mod run;
pub use run::*;

mod step;

mod binop;
pub use binop::*;

mod heur;
pub use heur::*;

mod gc;
pub use gc::*;

mod merge;
pub use merge::*;

mod subsumes;
pub use subsumes::*;

mod fmt;

// Tables from different TableSortIds are guaranteed to be distinct.
// Generally, TableSortIds work with weak updates.
// You need to wrap them in a ValueId for strong updates.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub struct TableSortId(pub Symbol);

// Represents a symbolic value. Different ValueIds can refer to the same value.
// In this system, you can never equate a ValueId to something (not even to other ValueIds).
// It will remain a distinct symbolical object.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ValueId(pub Symbol);

// Represents a specialization of some ProcId, based on some call context represented by "ThreadState".
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct SpecId(pub Symbol);

pub struct Homomorphism {
    pub m_value_id: Map<ValueId, ValueId>, // bijective map from a _subset_ of the inputs ValueIds to all of the outputs ValueIds.
    pub m_table_sort: Map<TableSortId, TableSortId>, // non-injective map from all inputs TableSortIds to all outputs TableSortIds.
}

#[derive(Debug)]
pub struct AnalysisState {
    pub root_spec: SpecId,
    pub specs: Map<SpecId, Spec>,
    pub queue: Vec<SpecId>, // these Specs still need to be computed.
    pub heur_queue: Vec<SpecId>, // these Specs still need to run through the heuristic.
    pub ir: IR,
}

#[derive(Debug)]
pub struct Spec {
    pub st: ThreadState,

    // we don't actually need to store the homomorphism here I think.
    // The mere existance of a homomorphism suffices.
    // But maybe the heuristic cares for that homomorphism?
    // Note for later, if we want the homomorphism, we'd need to store a collection of (ThreadState, Homomorphism, SpecId) triples,
    // otherwise it's unclear between which things you are declaring a homomorphism. (as the out ThreadState state uses fresh names)
    pub outs: Vec<SpecId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEntry {
    Clear(ValueSet, ValueSet),
    Add(ValueSet, ValueSet, ValueSet),
}

pub type Deref = Map<ValueId, ValueSet>;

#[derive(Clone, Debug)]
pub struct ThreadState {
    // TODO: add cache: Map<TableSortId, Vec<usize>>,
    // we might want something more address-stable than "usize" though!
    pub table_entries: Vec<TableEntry>,

    pub root: ValueId,
    pub pid: Symbol,

    pub deref: Deref,
    pub nodes: Map<Node, ValueParticle>,
}

impl ThreadState {
    #[track_caller]
    pub fn check(&self) {
        let mut vids = vec![self.root];
        let it1 = self.deref.values();
        let it2 = self.table_entries.iter().map(|x| {
            let slice = match x {
                TableEntry::Add(t, k, v) => vec![t, k, v], // TODO fix vec! waste.
                TableEntry::Clear(t, k) => vec![t, k],
            };
            slice.into_iter()
        }).flatten();

        let it = it1.chain(it2).map(|x| x.0.iter()).flatten();
        vids.extend(it.filter_map(ValueParticle::to_valueid));

        for x in vids {
            assert!(self.deref.contains_key(&x));
        }
    }
}
