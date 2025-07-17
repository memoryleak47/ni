use crate::*;

pub fn store_p(t: &ValueParticle, k: &ValueParticle, v: ValueSet, st: ThreadState) -> ThreadState {
    todo!()
/*
            let overlaps = st.ts_store()
            let t = tovs(t);
            let k = tovs(k);
            let v = tovs(v);

            // remove strictly overwritten stuff.
            st.tkvs.retain(|(t2, k2, _)| !t.concrete_eq(t2) || !k.concrete_eq(k2));

            for (t2, k2, v2) in st.tkvs.iter_mut() {
                if t.overlaps(&*t2, &st.deref) && k.overlaps(&*k2, &st.deref) {
                    *v2 = v2.union(&v);
                }
            }

            st.tkvs.push((t, k, v));

            vec![st]
*/
}
