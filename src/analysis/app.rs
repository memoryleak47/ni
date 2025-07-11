use crate::*;

struct AppliedSpecId {
    m: SlotMap,
    id: SpecId,
}

// ommited entries keep their names.
struct SlotMap {
    m: Map<Id, Id>,
}
