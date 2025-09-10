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
    let (st1, tid1) = pre_simplify(st1);
    let (st2, tid2) = pre_simplify(st2);
    assert_eq!(tid1, tid2);

    let mut out = st1.clone();
    out.table_entries.extend(st2.table_entries.clone());

    let g = build_groups(&out);

    // TODO find similarities between these groups, and unify TableSortIds based on that.

    gc_table_entries(&mut out);

    out
}

fn unify_tids(tid1: TableSortId, tid2: TableSortId, st: &mut ThreadState) {
    if tid1 == tid2 { return }
    let (tid1, tid2) = if tid1 > tid2 { (tid2, tid1) } else { (tid1, tid2) };

    for e in st.table_entries.iter_mut() {
        let vset: &mut [&mut ValueSet] = match e {
            TableEntry::Add(t, k, v) => &mut [t, k, v],
            TableEntry::Clear(t, k) => &mut [t, k]
        };

        for x in vset {
            for y in x.0.iter_mut() {
                let ValueParticle::TableSort(y) = y else { continue };
                if *y == tid2 { *y = tid1; }
            }
        }
    }
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
// Returns root TableSortId.
fn pre_simplify(st: &ThreadState) -> (ThreadState, TableSortId) {
    let [ValueParticle::TableSort(tid)] = *st.deref[&st.root].0 else { panic!() };

    let vids: Vec<_> = st.deref.keys().collect();

    let mut st = st.clone();

    for vid in vids {
        ts_deref_valueid(&mut st, *vid);
    }

    gc_table_entries(&mut st);

    (st, tid)
}
