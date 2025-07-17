use crate::*;

fn relevant_t_particles(t: &ValueParticle, st: &ThreadState) -> Set<ValueParticle> {
    t.deref(&st.deref).0.iter().filter_map(|x| match x {
        ValueParticle::TableSort(ts) => Some(st.ts_cache[ts].iter().cloned()),
        _ => None,
    }).flatten().collect()
}

pub fn store_p(t: ValueParticle, k: ValueParticle, v: ValueParticle, mut st: ThreadState) -> ThreadState {
    // 1. extend overlapping tkvs.
/*
    st.st_cache
    for (t2, k2, v2) in st.tkvs.iter_mut() {
        if t.overlaps(&*t2, &st.deref) && k.overlaps(&*k2, &st.deref) {
            *v2 = v2.union(&v);
        }
    }
*/

    // 2. add/overwrite (t, k) entry.
    let kv = st.tkvs.entry(t).or_insert(Default::default());
    kv.insert(k, ValueSet(vec![v]));

    st
}
