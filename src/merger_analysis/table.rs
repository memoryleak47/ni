use crate::merger_analysis::*;

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

    // TODO add optimization: we only need to consider entries covered by these newest two.
    gc_table_entries(&mut st);

    st
}

// If an Add has a future "Clear" or "Add" whose t and k (and v for add) are a superset, we can ignore the original Add.
// If a Clear has previous overlapping (in t and k) Adds, we can remove it.
// (We could remove more, but for now we don't).
// TODO: this is brutally inefficient. TableEntries need to be grouped much more.
// TODO: merge this with "gc_ts".
pub fn gc_table_entries(st: &mut ThreadState) {
    // We first remove all adds, and then clears,
    // as removing adds can cause the removal of clears.
    let mut i = 0;
    while i < st.table_entries.len() {
        let TableEntry::Add(t, k, v) = &st.table_entries[i] else { i += 1; continue };
        if add_relevant([t, k, v], i, st) {
            i += 1;
        } else {
            st.table_entries.remove(i);
        }
    }

    let mut i = 0;
    while i < st.table_entries.len() {
        let TableEntry::Clear(t, k) = &st.table_entries[i] else { i += 1; continue };
        if clear_relevant([t, k], i, st) {
            i += 1;
        } else {
            st.table_entries.remove(i);
        }
    }

}

// checks if add is covered by further add or clear
fn add_relevant([t, k, v]: [&ValueSet; 3], i: usize, st: &ThreadState) -> bool {
    if t.is_bottom() || k.is_bottom() || v.is_bottom() { return false }

    let d = &st.deref;
    for e in st.table_entries[(i+1)..].iter() {
        let covered = match e {
            TableEntry::Add(t2, k2, v2) => t.subseteq(t2, d) && k.subseteq(k2, d) && v.subseteq(v2, d),
            TableEntry::Clear(t2, k2) => t.subseteq(t2, d) && k.subseteq(k2, d),
        };
        if covered { return false }
    }
    true
}

// checks if clear has previous Add that overlaps it.
fn clear_relevant([t, k]: [&ValueSet; 2], i: usize, st: &ThreadState) -> bool {
    if t.is_bottom() || k.is_bottom() { return false }

    let d = &st.deref;
    for e in st.table_entries[0..i].iter() {
        if let TableEntry::Add(t2, k2, _) = e && t.overlaps(t2, d) && k.overlaps(k2, d) { return true }
    }
    false
}
