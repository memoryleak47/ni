use crate::standard_analysis::*;

pub fn tab_index(t: &ValueSet, k: &ValueSet, st: &ProcState) -> ValueSet {
    let mut vs = ValueSet::bottom();
    for t in &t.0 {
        for k in &k.0 {
            let tk = [t.clone(), k.clone()];
            let undef_v = ValueSet(vec![ValueParticle::Symbol(Symbol::new("Undef"))]);
            let vv = st.tables.get(&tk).cloned().unwrap_or(undef_v);
            vs.0.extend(vv.0);
        }
    }
    vs.compactify()
}

pub fn tab_store(t: &ValueSet, k: &ValueSet, v: &ValueSet, st: &mut ProcState) {
    let concrete = t.is_concrete() && k.is_concrete();
    for t in &t.0 {
        if !matches!(t, ValueParticle::Concrete(_)|ValueParticle::Summary(_)) { continue }
        for k in &k.0 {
            store_impl(t, k, v, concrete, st);
            if let ValueParticle::String(_) = k {
                store_impl(t, &ValueParticle::TopString, v, false, st);
            } else if let ValueParticle::Int(_) = k {
                store_impl(t, &ValueParticle::TopInt, v, false, st);
            }
        }
    }
}

fn store_impl(t: &ValueParticle, k: &ValueParticle, v: &ValueSet, concrete: bool, st: &mut ProcState) {
    let tk = [t.clone(), k.clone()];
    let vv = st.tables.entry(tk).or_insert_with(|| ValueSet(vec![ValueParticle::Symbol(Symbol::new("Undef"))]));
    if concrete {
        *vv = v.clone();
    } else {
        *vv = vv.union(v);
    }
}
