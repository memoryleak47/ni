use crate::*;

type Cache = Map<(ValueParticle, ValueParticle), ValueSet>;

pub fn upcast(p: &ValueParticle, deref: &Deref) -> Option<ValueSet> {
    match p {
        ValueParticle::ValueId(v) => deref.get(v).cloned(),
        ValueParticle::String(_) => Some(ValueSet(vec![ValueParticle::TopString])),
        ValueParticle::Int(_) => Some(ValueSet(vec![ValueParticle::TopInt])),
        ValueParticle::Top => None,
        _ => Some(ValueSet(vec![ValueParticle::Top])),
    }
}

pub fn index_p(t: &ValueParticle, k: &ValueParticle, st: &ThreadState, m: &mut Cache) -> ValueSet {
    // 1. check the cache.
    let tk = (t.clone(), k.clone());
    if let Some(x) = m.get(&tk) { return x.clone(); }

    // 2. check tkvs
    if let Some(kv) = st.tkvs.get(t) {
        if let Some(v) = kv.get(k) {
            m.insert(tk, v.clone());
            return v.clone();
        }
    }

    // 3.1 try to upcast
    let t_set = ValueSet(vec![t.clone()]);
    let k_set = ValueSet(vec![k.clone()]);

    let v1 = upcast(t, &st.deref).map(|t_upc| index(&t_upc, &k_set, st, m));
    let v2 = upcast(k, &st.deref).map(|k_upc| index(&t_set, &k_upc, st, m));

    let v = match (v1, v2) {
        (Some(v1), Some(v2)) => intersect(v1, v2, &st.deref),
        (Some(v), None) | (None, Some(v)) => v,
        (None, None) => ValueSet(vec![ValueParticle::Top]),
    };

    // 3.2 intersect with weak matches.
    let v_weak = index_p_weak(t, k, st);
    let v = intersect(v, v_weak, &st.deref);
    m.insert(tk, v.clone());
    v
}

fn index_p_weak(t: &ValueParticle, k: &ValueParticle, st: &ThreadState) -> ValueSet {
    let mut out = ValueSet(Vec::new());
    out.0.push(ValueParticle::Symbol(Symbol::new("Undef")));

    for (t2, kv2) in st.tkvs.iter() {
        if t.overlaps(&*t2, &st.deref) {
            for (k2, v2) in kv2.iter() {
                if k.overlaps(&*k2, &st.deref) {
                    out = out.union(&v2, &st.deref);
                }
            }
        }
    }

    out
}

fn index(t: &ValueSet, k: &ValueSet, st: &ThreadState, m: &mut Cache) -> ValueSet {
    // this function implements the heap laws:
    // - index(t1 \/ t2, k) = index(t1, k) \/ index(t2, k)
    // - index(t, k1 \/ k2) = index(t, k1) \/ index(t, k2)

    let mut out = ValueSet(Vec::new());
    for tt in &t.0 {
        for kk in &k.0 {
            out = out.union(&index_p(tt, kk, st, m), &st.deref);
        }
    }
    out
}

// this is overapproximating an intersection!
pub fn intersect(a: ValueSet, b: ValueSet, deref: &Deref) -> ValueSet {
    // distributivity:
    // (a1 \/ a2) /\ b = (a1 /\ b) \/ (a2 /\ b)
    // a /\ (b1 \/ b2) = (a /\ b1) \/ (a /\ b2)

    let mut out = ValueSet(Vec::new());
    for aa in &a.0 {
        for bb in &b.0 {
            out = out.union(&intersect_p(aa, bb, deref), deref);
        }
    }
    out
}

pub fn intersect_p(a: &ValueParticle, b: &ValueParticle, deref: &Deref) -> ValueSet {
    use ValueParticle::*;
    match (a, b) {
        (Top, o) | (o, Top) => ValueSet(vec![o.clone()]),
        (v@ValueId(_), o) | (o, v@ValueId(_)) => {
            // If we want a strict "intersect", this would be a "subseteq" check, but then we might underapproximate.
            let cond = v.overlaps(&o, deref);
            if cond { ValueSet(vec![v.clone()]) } else { ValueSet(Vec::new()) }
        },
        (s@String(_), TopString) | (TopString, s@String(_)) => ValueSet(vec![s.clone()]),
        (i@Int(_), TopInt) | (TopInt, i@Int(_)) => ValueSet(vec![i.clone()]),
        (x, y) if x == y => ValueSet(vec![x.clone()]),
        _ => ValueSet(Vec::new()),
    }
}
