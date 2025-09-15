use crate::*;

// phi(a) <= heap_general(phi(t), phi(k))
// Note: for non-TIDs x, phi(x) = x.
struct Constraint {
    a: ValueParticle,
    t: ValueParticle,
    k: ValueParticle,
}

type Phi = Map<TableSortId, Set<TableSortId>>;

type Homomorphism = Map<TableSortId, TableSortId>;

pub fn subsumes2(general: &ThreadState, special: &ThreadState) -> bool {
    if general.pid != special.pid { return false; }

    let (special, _) = pre_simplify(special);
    let (general, _) = pre_simplify(general);

    let constraints = build_constraints(&special);

    let tids_special = tids(&special);
    let tids_general = tids(&general);

    let phi = tids_special.iter().map(|x| (*x, tids_general.clone())).collect();
    solve_constraints(phi, constraints, &general).is_some()
}

fn build_constraints(special: &ThreadState) -> Vec<Constraint> {
    let mut constraints = Vec::new();
    for e in &special.table_entries {
        let TableEntry::Add(t, k, v) = e else { continue };
        for t in &t.0 {
            for k in &k.0 {
                for a in &v.0 {
                    let a = a.clone();
                    let t = t.clone();
                    let k = k.clone();
                    constraints.push(Constraint { a, t, k });
                }
            }
        }
    }
    constraints
}

fn solve_constraints(phi: Phi, constraints: Vec<Constraint>, general: &ThreadState) -> Option<Homomorphism> {
    todo!()
}

fn done(phi: &Phi) -> bool {
    phi.iter().all(|(_, x)| x.len() == 1)
}

fn eval_phi(p: &ValueParticle, phi: &Phi) -> ValueSet {
    if let ValueParticle::TableSort(tid) = p {
        ValueSet(phi[tid].iter().cloned().map(ValueParticle::TableSort).collect())
    } else {
        ValueSet(vec![p.clone()])
    }
}

fn constraints_satisfiable(phi: &Phi, constraints: &[Constraint], general: &ThreadState) -> bool {
    for Constraint { a, t, k } in constraints {
        let t = eval_phi(t, phi);
        let k = eval_phi(k, phi);
        let a = eval_phi(a, phi);
        let v = index(&t, &k, general);
        let cond = a.0.iter().any(|a| {
            a.subseteq(&v, &general.deref)
        });
        if !cond { return false }
    }

    true
}

fn index(t: &ValueSet, k: &ValueSet, st: &ThreadState) -> ValueSet {
    let mut vs = ValueSet::bottom();
    for t in &t.0 {
        for k in &k.0 {
            vs = vs.union(&index_p(t, k, st), &st.deref);
        }
    }
    vs
}

fn get_support(st: &ThreadState) -> Vec<ValueParticle> {
    let mut sup: Set<ValueParticle> = Set::new();
    for (_, vs) in st.deref.iter() {
        sup.extend(vs.0.iter().cloned());
    }

    for e in st.table_entries.iter() {
        match e {
            TableEntry::Add(t, k, v) => {
                sup.extend(t.0.iter().cloned());
                sup.extend(k.0.iter().cloned());
                sup.extend(v.0.iter().cloned());
            },
            TableEntry::Clear(t, k) => {
                sup.extend(t.0.iter().cloned());
                sup.extend(k.0.iter().cloned());
            },
        }
    }
    sup.into_iter().collect()
}

fn tids(st: &ThreadState) -> Set<TableSortId> {
    let mut set = Set::new();
    for (_, x) in &st.deref {
        set.extend(x.0.iter().filter_map(ValueParticle::to_tid));
    }
    for e in &st.table_entries {
        match e {
            TableEntry::Add(t, k, v) => {
                set.extend(t.0.iter().filter_map(ValueParticle::to_tid));
                set.extend(k.0.iter().filter_map(ValueParticle::to_tid));
                set.extend(v.0.iter().filter_map(ValueParticle::to_tid));
            },
            TableEntry::Clear(t, k) => {
                set.extend(t.0.iter().filter_map(ValueParticle::to_tid));
                set.extend(k.0.iter().filter_map(ValueParticle::to_tid));
            },
        }
    }
    set
}
