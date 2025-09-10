use crate::*;

#[derive(Eq, Hash, PartialEq)]
enum ValueGroup {
    Table(TableSortId),
    Symbol(Symbol),
    TopString,
    TopInt,
}

type Groups = Map<(TableSortId, ValueGroup), Vec<usize>>;

pub fn merge(st1: &ThreadState, st2: &ThreadState) -> ThreadState {
    let st1 = pre_simplify(st1);
    let st2 = pre_simplify(st2);

    let g1 = build_groups(&st1);
    let g2 = build_groups(&st2);

    // TODO find similarities between these groups, and unify TableSortIds based on that.

    todo!()
}

fn build_groups(st: &ThreadState) -> Groups {
    let mut groups: Groups = Groups::new();

    for (i, e) in st.table_entries.iter().enumerate() {
        let TableEntry::Add(t, k, v) = e else { continue };
        for t in &t.0 {
            let ValueParticle::TableSort(t) = *t else { continue };
            for k in &k.0 {
                let k = groupify(k);
                groups.entry((t, k)).or_insert_with(Default::default).push(i);
            }
        }
    }

    groups
}

fn groupify(p: &ValueParticle) -> ValueGroup {
    match p {
        ValueParticle::Top => panic!("How did Top get here?"),
        ValueParticle::Symbol(s) => ValueGroup::Symbol(*s),
        ValueParticle::TopString | ValueParticle::String(_) => ValueGroup::TopString,
        ValueParticle::TopInt | ValueParticle::Int(_) => ValueGroup::TopInt,
        ValueParticle::TableSort(tid) => ValueGroup::Table(*tid),
        ValueParticle::ValueId(_) => unreachable!(),
    }
}

// gets rid of all ValueIds, and thus clears for now.
fn pre_simplify(st: &ThreadState) -> ThreadState {
    let vids: Vec<_> = st.deref.keys().collect();

    let mut st = st.clone();

    for vid in vids {
        ts_deref_valueid(&mut st, *vid);
    }

    gc_table_entries(&mut st);

    st
}
