use crate::*;

pub struct Homomorphism {
    // map from _all_ general value ids to _some_ special value ids.

    // might not be injective, as in the special case some ValueIds can be unioned.
    // NOTE: this makes most sense, if all constants are also covered by ValueIds.

    // also: this function might not be surjective, as some ValueIds from the special case might be forgotten in the general case.
    value_id_map: Map<ValueId, ValueId>,

    // map from _all_ special TableSortIds to _all_ general TableSortIds (total).
    // not generally injective, as we merge table sorts during widening.
    ts_map: Map<TableSortId, TableSortId>,
}

impl AnalysisState {
    pub fn widen(&mut self, special: SpecId, general: SpecId, hom: Homomorphism) {
        assert!(self.check_hom(special, general, hom));

        for x in self.queue.iter_mut() {
            if *x == special { *x = general; }
        }
    }

    fn check_hom(&self, special: SpecId, general: SpecId, hom: Homomorphism) -> bool {
        todo!()
    }
}
