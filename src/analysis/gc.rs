use crate::*;

pub fn gc_ts(st: &mut ThreadState) {
    // trivial GC that only understands empty vs. non-empty states.
    let empty = st.table_entries.iter().all(|x| {
        if let TableEntry::Add(t, _, _) = x {
            !t.overlaps(&ValueSet(vec![ValueParticle::ValueId(st.root)]), &st.deref)
        } else { true }
    });
    if empty {
        st.table_entries.clear();
        let r = st.deref[&st.root].clone();
        st.deref.clear();
        st.deref.insert(st.root.clone(), r);
    }
}
