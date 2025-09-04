use crate::*;

pub fn gc_ts(st: &mut ThreadState) {
    let reach = reachable_table_sorts(st);

    // "deref away" unreachable ValueIds (as implied by `reach`).
    for (vid, vs) in st.deref.clone() {
        let live = has_free_symbol(&vs) || !get_table_sorts(&vs).is_disjoint(&reach);
        if !live {
            ts_deref_valueid(st, vid);
        }
    }

    // eliminate non-reachable TableSortIds from the table-entries.
    for e in st.table_entries.iter_mut() {
        let slice: &mut [_] = match e {
            TableEntry::Add(t, k, v) => &mut [t, k, v],
            TableEntry::Clear(t, k) => &mut [t, k],
        };
        for x in slice {
            x.0.retain(|a| match a {
                ValueParticle::TableSort(tid) => reach.contains(tid),
                _ => true,
            });
        }
    }

    // clear empty table entries.
    st.table_entries.retain(|e| {
        match e {
            TableEntry::Add(t, k, v) => t.0.len() > 0 && k.0.len() > 0 && v.0.len() > 0,
            TableEntry::Clear(t, k) => t.0.len() > 0 && k.0.len() > 0,
        }
    });
}

// The correct way to remove a ValueId from a ThreadState.
pub fn ts_deref_valueid(st: &mut ThreadState, vid: ValueId) {
    let replacement = st.deref.remove(&vid).unwrap();
    let deref = st.deref.clone();

    for (_, vs) in st.deref.iter_mut() {
        replace_vid(vs, vid, &replacement, &deref);
    }

    for e in st.table_entries.iter_mut() {
        match e {
            TableEntry::Add(t, k, v) => {
                // We replace as we want to overapproximate "Add".
                replace_vid(t, vid, &replacement, &deref);
                replace_vid(k, vid, &replacement, &deref);
                replace_vid(v, vid, &replacement, &deref);
            },
            TableEntry::Clear(t, k) => {
                // We remove as we want to underapproximate "Clear".
                remove_vid(t, vid);
                remove_vid(k, vid);
            },
        }
    }
}

fn remove_vid(vs: &mut ValueSet, vid: ValueId) {
    vs.0.retain(|x| *x != ValueParticle::ValueId(vid));
}

fn replace_vid(vs: &mut ValueSet, vid: ValueId, replacement: &ValueSet, deref: &Deref) {
    if let Some(i) = vs.0.iter().position(|x| *x == ValueParticle::ValueId(vid)) {
        vs.0.swap_remove(i);
        *vs = vs.union(replacement, deref);
    }
}

fn reachable_table_sorts(st: &ThreadState) -> Set<TableSortId> {
    let mut known: Set<TableSortId> = get_table_sorts(&ValueParticle::ValueId(st.root).deref(&st.deref));

    loop {
        let n = known.len();

        for entry in st.table_entries.iter() {
            let TableEntry::Add(t, k, v) = entry else { continue };
            let t_deref = t.deref(&st.deref);
            let k_deref = k.deref(&st.deref);
            let v_deref = v.deref(&st.deref);

            let t = get_table_sorts(&t_deref);
            let k = get_table_sorts(&k_deref);
            let v = get_table_sorts(&v_deref);

            let t_fine = !t.is_disjoint(&known);
            let k_fine = has_free_symbol(&k_deref) || !k.is_disjoint(&known);
            if t_fine && k_fine {
                known.extend(v);
            }
        }

        if n == known.len() { break }
    }

    known
}

// input is required to be "dereffed".
fn get_table_sorts(vs: &ValueSet) -> Set<TableSortId> {
    let mut out = Set::new();
    for x in vs.0.iter() {
        if let ValueParticle::TableSort(tid) = x {
            out.insert(*tid);
        }
    }
    out
}

fn has_free_symbol(vs: &ValueSet) -> bool {
    for x in vs.0.iter() {
        match x {
            ValueParticle::Top
            | ValueParticle::TopString
            | ValueParticle::String(_)
            | ValueParticle::TopInt
            | ValueParticle::Int(_)
            | ValueParticle::Symbol(_) => return true,

            ValueParticle::TableSort(_)
            | ValueParticle::ValueId(_) => {},
        }
    }

    false
}

