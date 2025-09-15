use crate::*;

// phi(a) <= heap_general(phi(t), phi(k))
// Note: for non-TIDs x, phi(x) = x.
struct Constraint {
    a: ValueParticle,
    t: ValueParticle,
    k: ValueParticle,
}

type Homomorphism = Map<TableSortId, TableSortId>;

pub fn subsumes2(general: &ThreadState, special: &ThreadState) -> bool {
    if general.pid != special.pid { return false; }

    let (special, _) = pre_simplify(special);
    let (general, _) = pre_simplify(general);

    let constraints = build_constraints(&special);

    solve_constraints(constraints, &general).is_some()
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

fn solve_constraints(constraints: Vec<Constraint>, general: &ThreadState) -> Option<Homomorphism> {
    todo!()
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

