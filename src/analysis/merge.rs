use crate::*;

pub fn merge(st1: &ThreadState, st2: &ThreadState) -> ThreadState {
    let st1 = pre_simplify(st1);
    let st2 = pre_simplify(st2);

    todo!()
}

// gets rid of all ValueIds, and thus clears for now.
fn pre_simplify(st: &ThreadState) -> ThreadState {
    let vids: Vec<_> = st.deref.keys().collect();

    let mut st = st.clone();

    for vid in vids {
        ts_deref_valueid(&mut st, *vid);
    }
    st
}
