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

// TODO: upcast "k".
pub fn tab_store(t: &ValueSet, k: &ValueSet, v: &ValueSet, st: &mut ProcState) {
    let concrete = t.is_concrete() && k.is_concrete();
    for t in &t.0 {
        assert!(matches!(t, ValueParticle::Concrete(_)|ValueParticle::Summary(_)));

        for k in &k.0 {
            let tk = [t.clone(), k.clone()];
            let vv = st.tables.entry(tk).or_default();
            if concrete {
                *vv = v.clone();
            } else {
                let undef_v = ValueSet(vec![ValueParticle::Symbol(Symbol::new("Undef"))]);
                *vv = vv.union(v).union(&undef_v);
            }
        }
    }
}
