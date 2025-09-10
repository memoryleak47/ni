use crate::*;

#[derive(Eq, Hash, PartialEq)]
enum ValueGroup {
    Table(TableSortId),
    Symbol(Symbol),
    TopString,
    TopInt,
}

type Groups = Map<(TableSortId, ValueGroup), Set<TableSortId>>;

pub fn merge(st1: &ThreadState, st2: &ThreadState) -> ThreadState {
    st1.check();
    st2.check();
    let (st1, tid1) = pre_simplify(st1);
    let (st2, tid2) = pre_simplify(st2);
    assert_eq!(st1.root, st2.root);
    assert_eq!(tid1, tid2);

    let mut out = st1.clone();
    out.table_entries.extend(st2.table_entries.clone());

    loop {
        gc_table_entries(&mut out);

        let mut something_happened = false;

        let mut tab_intersection: Map<[TableSortId; 2], usize> = Map::new();
        let mut tab_count: Map<TableSortId, usize> = Map::new();

        for (_, tids) in build_groups(&out) {
            for &tid in &tids { *tab_count.entry(tid).or_insert(0) += 1; }
            for &tid1 in &tids {
                for &tid2 in &tids {
                    if tid1 < tid2 {
                        *tab_intersection.entry([tid1, tid2]).or_insert(0) += 1;
                    }
                }
            }
        }

        for ([tid1, tid2], intersection_ctr) in tab_intersection {
            let union_ctr = tab_count[&tid1] + tab_count[&tid2] - intersection_ctr;
            let iou = intersection_ctr as f64 / union_ctr as f64;
            if iou > 0.5 {
                unify_tids(tid1, tid2, &mut out);
            }
        }
        if !something_happened {
            break;
        }
    }

    out.deref.insert(st1.root, ValueSet(vec![ValueParticle::TableSort(tid1)]));

    gc_table_entries(&mut out);

    out.check();
    out
}

fn unify_tids(tid1: TableSortId, tid2: TableSortId, st: &mut ThreadState) {
    if tid1 == tid2 { return }
    let (tid1, tid2) = if tid1 > tid2 { (tid2, tid1) } else { (tid1, tid2) };

    for e in st.table_entries.iter_mut() {
        let vset: &mut [&mut ValueSet] = match e {
            TableEntry::Add(t, k, v) => &mut [t, k, v],
            TableEntry::Clear(t, k) => &mut [t, k] // TODO is this correct, even for clear?
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

    for e in st.table_entries.iter() {
        let TableEntry::Add(t, k, v) = e else { continue };
        for t in &t.0 {
            let ValueParticle::TableSort(t) = *t else { continue };
            for k in &k.0 {
                let k = groupify(k);
                let tids = v.0.iter().filter_map(ValueParticle::to_tid);
                groups.entry((t, k)).or_insert_with(Default::default).extend(tids);
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
