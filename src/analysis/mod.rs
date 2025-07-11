mod app;
mod valueset;

// represents a concrete symbolic object.
// different ValueIds can refer to the same object. Once this is detected, we'll typically union them.
type ValueId = Id;

// Tables from different TableSortIds are guaranteed to be distinct.
// Generally, TableSortIds work with weak updates.
// You need to wrap it in a ValueId to make it concrete.
type TableSortId = Id;

// a proc specialization.
type SpecId = Id;

struct AnalysisState {
  specs: SpecId -> Spec,
  queue: Queue<SpecId>, // these SpecIds still need to be computed.
}

struct Spec {
  st: ThreadState,
  out: Set<AppliedSpecId>,
}

struct ThreadState {
    // // maybe later:
    // facts: Set<Fact>,
    // // for general facts about ValueIds, in particular ints & strings and which ValueIds are ops computed from others...
    // // another typical fact (as seen in the nondet example from before), is to assert that two ValueIds are not equal.

    // forall (t: T), forall (k: K), exists (v: V), t[k] = v.
    // any entry in T is a TableSortId, or a ValueId recursively refering to one.
    tkvs: Set<(T: ValueSet, K: ValueSet, V: ValueSet)>,

    deref_val_id: ValueId -> ValueSet,

    root: ValueId,

    pid: ProcId,
}
