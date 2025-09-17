use crate::standard_analysis::*;

pub fn heur(a: &mut AnalysisState, new: SpecId) {
    assert!(a.specs.contains_key(&new));

    if let Some(o) = is_subsumed(a, new) {
        replace_a(new, o, a);
        gc_a(a);
    } else {
        a.queue.push(new);

        // checks for whether we want to call a "merge".
        let pid = a.specs[&new].st.pid;
        if a.specs.iter().filter(|(_, x)| x.st.pid == pid).count() >= 50 {
            // merge the last two ones.
            let [(&id1, st1), (&id2, st2)] = *a.specs.iter().filter(|(_, x)| x.st.pid == pid).rev().take(2).collect::<Vec<_>>() else { panic!() };

            let new = merge(&st1.st, &st2.st);
            if CHECKS {
                assert!(subsumes(&new, &st1.st));
                assert!(subsumes(&new, &st2.st));
            }
            let new = a.add(new);
            replace_a(id1, new, a);
            replace_a(id2, new, a);
            gc_a(a);
        }
    }
}

fn is_subsumed(a: &AnalysisState, new: SpecId) -> Option<SpecId> {
    let st = &a.specs[&new].st;
    for (o, st_o) in a.specs.iter() {
        if *o != new && subsumes(&st_o.st, st) {
            return Some(*o);
        }
    }
    None
}

fn replace_a(bad: SpecId, good: SpecId, a: &mut AnalysisState) {
    assert!(bad != good);

    a.specs.swap_remove(&bad);

    for (_, x) in a.specs.iter_mut() {
        for d in x.outs.iter_mut() {
            if *d == bad {
                *d = good;
            }
        }
    }

    let n = a.queue.len();
    a.queue.retain(|d| *d != bad);
    if a.queue.len() != n && !a.queue.contains(&good) {
        a.queue.push(good);
    }

    let n = a.heur_queue.len();
    a.heur_queue.retain(|d| *d != bad);
    if a.heur_queue.len() != n && !a.heur_queue.contains(&good) {
        a.heur_queue.push(good);
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
    a.heur_queue.retain(|k| known.contains(k));
}
