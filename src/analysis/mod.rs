use crate::*;

mod app;
pub use app::*;

mod valueset;
pub use valueset::*;

mod run;
pub use run::*;

mod index;
pub use index::*;

mod store;
pub use store::*;

mod step;
pub use step::*;

mod binop;
pub use binop::*;

// Tables from different TableSortIds are guaranteed to be distinct.
// Generally, TableSortIds work with weak updates.
// You need to wrap them in a ValueId for strong updates.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
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

pub struct AnalysisState {
    pub specs: Map<SpecId, Spec>,
    pub queue: Vec<SpecId>, // these Specs still need to be computed.
    pub ir: IR,
}

pub struct Spec {
    pub st: ThreadState,

    // we don't actually need to store the homomorphism here I think.
    // The mere existance of a homomorphism suffices.
    // But maybe the heuristic cares for that homomorphism?
    // Note for later, if we want the homomorphism, we'd need to store a collection of (ThreadState, Homomorphism, SpecId) triples,
    // otherwise it's unclear between which things you are declaring a homomorphism. (as the out ThreadState state uses fresh names)
    pub outs: Vec<SpecId>,
}

pub type Deref = Map<ValueId, ValueSet>;

#[derive(Clone)]
pub struct ThreadState {
    // semantics: all tkv-triples are universally true (they hold for all runtime values contained in these t,k-ValueSets),
    // thus we intersect over all possibilities (while using disjunctive heap laws to get rid of ValueSets).
    pub tkvs: Map</*T*/ ValueParticle, Map</*K*/ ValueParticle, /*V*/ ValueSet>>,

    // ts_cache[t] = {t' ValueParticle | t overlaps t'}
    pub ts_cache: Map<TableSortId, Vec<ValueParticle>>,

    pub root: ValueId,
    pub pid: Symbol,

    pub deref: Deref,
    pub nodes: Map<Node, ValueParticle>,
}
