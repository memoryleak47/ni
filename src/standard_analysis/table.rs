use crate::standard_analysis::*;

pub fn tab_index(t: &ValueSet, k: &ValueSet, st: &ProcState) -> ValueSet {
    let mut vs = ValueSet::bottom();
    for t in &t.0 {
        for k in &k.0 {
            vs.0.extend(st.tables[&[t.clone(), k.clone()]].0.clone());
        }
    }
    vs.compactify()
}

pub fn tab_store(t: &ValueSet, k: &ValueSet, v: &ValueSet, st: &mut ProcState) {
    for t in &t.0 {
        for k in &k.0 {
            let vv = &mut st.tables[&[t.clone(), k.clone()]];
            *vv = vv.union(v);
        }
    }
}
