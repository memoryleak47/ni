use crate::*;

fn relevant_t_particles(t: &ValueParticle, st: &ThreadState) -> Set<ValueParticle> {
    t.deref(&st.deref).0.iter().filter_map(|x| match x {
        ValueParticle::TableSort(ts) => Some(st.ts_cache[ts].iter().cloned()),
        _ => None,
    }).flatten().collect()
}

pub fn store_p(t: ValueParticle, k: ValueParticle, v: ValueParticle, mut st: ThreadState) -> ThreadState {
    let v = ValueSet(vec![v]);

    // 1. extend superset stuff.
    extend_all(&t, &k, &mut st);
    // NOTE: when you write to t[%a], and then load from t[%b] (where deref(a) = deref(b) = TopString), the system might not understand that this store & load does overlap.
    // Why, because adding the store simply adds a tkv entry; and computing t[%b] only looks for ancestor entries, so it will miss the t[%a] one!
    // For that reason, we force all ancestor entries to exist, so that we can update them in the "overlaps" loop below.

    // 2. extend overlapping tkvs.

    // TODO optimization: use relevant_t_particles to restrict the search space.

    for (t2, kv2) in st.tkvs.iter_mut() {
        if t.overlaps(&*t2, &st.deref) {
            for (k2, v2) in kv2.iter_mut() {
                if k.overlaps(&*k2, &st.deref) {
                    *v2 = v2.union(&v, &st.deref);
                }
            }
        }
    }

    // 3. add/overwrite (t, k) entry.
    let kv = st.tkvs.entry(t).or_insert(Default::default());
    kv.insert(k, v);

    st
}

// adds all tkv entries (T, K) where t <= T and k <= K.
fn extend_all(t: &ValueParticle, k: &ValueParticle, st: &mut ThreadState) {
    let v = index_p(t, k, st, &mut Default::default());
    let entry = st.tkvs.entry(t.clone()).or_insert(Default::default());
    entry.insert(k.clone(), v);

    if let Some(t) = upcast(t, &st.deref) {
        for t in &t.0 {
            extend_all(t, k, st);
        }
    }

    if let Some(k) = upcast(k, &st.deref) {
        for k in &k.0 {
            extend_all(t, k, st);
        }
    }
}
