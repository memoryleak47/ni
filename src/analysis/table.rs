use crate::*;

pub fn index_p(t: &ValueParticle, k: &ValueParticle, st: &ThreadState) -> ValueSet {
    let t = ValueSet([t.clone()].into_iter().collect());
    let k = ValueSet([k.clone()].into_iter().collect());

    let mut vs = ValueSet(Default::default());
    for entry in st.table_entries.iter().rev() {
        match entry {
            TableEntry::Clear(t1, k1) => {
                if t.subseteq(&t1, &st.deref) && k.subseteq(&k1, &st.deref) { break; }
            },
            TableEntry::Add(t1, k1, v1) => {
                if t1.overlaps(&t, &st.deref) && k1.overlaps(&k, &st.deref) {
                    vs = vs.union(v1, &st.deref);
                }
            },
        }
    }
    vs
}

pub fn store_p(t: ValueParticle, k: ValueParticle, v: ValueParticle, mut st: ThreadState) -> ThreadState {
    let t = ValueSet([t].into_iter().collect());
    let k = ValueSet([k].into_iter().collect());
    let v = ValueSet([v].into_iter().collect());
    st.table_entries.push(TableEntry::Clear(t.clone(), k.clone()));
    st.table_entries.push(TableEntry::Add(t, k, v));
    st
}
