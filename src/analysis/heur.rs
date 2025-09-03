use crate::*;

// only allowed to mutate using a.widen() and a.add()
pub fn heur(a: &mut AnalysisState, new: SpecId) {
    let st = &a.specs[&new].st;
    for (o, st_o) in a.specs.iter() {
        if *o != new && subsumes(&st_o.st, st) {
            replace_a(new, *o, a);
            gc_a(a);
            return;
        }
    }
}

fn subsumes(general: &ThreadState, special: &ThreadState) -> bool {
    // TODO trivial subsumption check.
    general.pid == special.pid && general.table_entries.len() == 0 && special.table_entries.len() == 0
}

fn replace_a(bad: SpecId, good: SpecId, a: &mut AnalysisState) {
    a.specs.remove(&bad);

    for (_, x) in a.specs.iter_mut() {
        for d in x.outs.iter_mut() {
            if *d == bad {
                *d = good;
            }
        }
    }

    let n = a.queue.len();
    a.queue.retain(|d| *d != bad);
    if a.queue.len() != n {
        a.queue.push(good);
    }
}

fn gc_a(a: &mut AnalysisState) {
    let mut known = Set::new();
    let mut queue = Vec::new();
    known.insert(a.root_spec);
    queue.push(a.root_spec);

    while let Some(x) = queue.pop() {
        for d in a.specs[&x].outs.iter() {
            if !known.contains(d) {
                known.insert(*d);
                queue.push(*d);
            }
        }
    }

    a.specs.retain(|k, _| known.contains(k));
    a.queue.retain(|k| known.contains(k));
}
