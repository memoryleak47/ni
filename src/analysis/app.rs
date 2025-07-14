use crate::*;

pub struct AppliedSpecId {
    pub m: SlotMap,
    pub id: SpecId,
}

// ommited entries keep their names.
pub struct SlotMap {
    m: Map<Id, Id>,
}
