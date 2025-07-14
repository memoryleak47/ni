use crate::*;

use std::collections::VecDeque;

mod app;
pub use app::*;

mod valueset;
pub use valueset::*;

mod run;
pub use run::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Id(pub usize);

// represents a concrete symbolic object.
// different ValueIds can refer to the same object. Once this is detected, we'll typically union them.
pub type ValueId = Id;

// Tables from different TableSortIds are guaranteed to be distinct.
// Generally, TableSortIds work with weak updates.
// You need to wrap it in a ValueId to make it concrete.
pub type TableSortId = Id;

pub type ProcId = Symbol;

// a proc specialization.
pub type SpecId = Id;

pub struct AnalysisState {
    pub specs: Map<SpecId, Spec>,
    pub queue: VecDeque<SpecId>, // these SpecIds still need to be computed.
}

pub struct Spec {
    pub st: ThreadState,
    pub outs: Vec<AppliedSpecId>, // set of output options.
}

pub struct ThreadState {
    // forall (t: T), forall (k: K), exists (v: V), t[k] = v.
    // any entry in T is a TableSortId, or a ValueId recursively refering to one.
    // TODO what's the story with overlapping t or k triples?
    tkvs: Set<(ValueSet, ValueSet, ValueSet)>,

    deref_val_id: Map<ValueId, ValueSet>,
    root: ValueId,
    pid: ProcId,

    // // maybe later:
    // facts: Set<Fact>,
    // // for general facts about ValueIds, in particular ints & strings and which ValueIds are ops computed from others...
    // // another typical fact (as seen in the nondet example from before), is to assert that two ValueIds are not equal.
}
