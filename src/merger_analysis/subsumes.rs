use crate::merger_analysis::*;

// We assume that TableSortIds coming up in general and special are mapped as the identity.
const ASSUME_IDENTITY: bool = true;

// phi(a) <= heap_general(phi(t), phi(k))
// Note: for non-TIDs x, phi(x) = x.
struct Constraint {
    a: ValueParticle,
    t: ValueParticle,
    k: ValueParticle,
}

type Phi = Map<TableSortId, Set<TableSortId>>;

type Homomorphism = Map<TableSortId, TableSortId>;

pub fn subsumes(general: &ThreadState, special: &ThreadState) -> bool {
    if general.pid != special.pid { return false; }

    let (special, _) = pre_simplify(special);
    let (general, _) = pre_simplify(general);

    let constraints = build_constraints(&special);

    let tids_special = tids(&special);
    let mut tids_general = tids(&general);
    // Incase tids_general is empty, we want some place to map everything to.
    tids_general.insert(TableSortId(Symbol::new_fresh("DEFAULT_TID".to_string())));

    let cache = heap_cache(&general);

    let phi = tids_special.iter().map(|x|
            if ASSUME_IDENTITY && tids_general.contains(x) { (*x, std::iter::once(*x).collect()) }
            else { (*x, tids_general.clone()) }
        ).collect();
    let out = solve_constraints(phi, &constraints, &general, &cache).is_some();
    out
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

fn solve_constraints(mut phi: Phi, constraints: &[Constraint], general: &ThreadState, cache: &HeapCache) -> Option<Homomorphism> {
    assert!(phi.iter().all(|(_, y)| y.len() > 0));
    if !constraints_satisfiable(&phi, constraints, general, cache) { return None; }

    if let Some((x, y)) = phi.iter().find(|(_, y)| y.len() > 1).map(|(x, y)| (x.clone(), y.clone())) {
        let mut phi_new = phi.clone();
        let y1 = y.iter().next().unwrap().clone();
        phi_new.insert(x, std::iter::once(y1).collect());
        if let Some(hom) = solve_constraints(phi_new, constraints, general, cache) {
            return Some(hom);
        }
        phi[&x].swap_remove(&y1);

        solve_constraints(phi, constraints, general, cache)
    } else {
        let hom: Homomorphism = phi.iter().map(|(x, y)| {
            assert!(y.len() == 1);
            let y = y.iter().next().unwrap().clone();
            (*x, y)
        }).collect();
        Some(hom)
    }
}

fn eval_phi(p: &ValueParticle, phi: &Phi) -> ValueSet {
    if let ValueParticle::TableSort(tid) = p {
        ValueSet(phi[tid].iter().cloned().map(ValueParticle::TableSort).collect())
    } else {
        ValueSet(vec![p.clone()])
    }
}

fn constraints_satisfiable(phi: &Phi, constraints: &[Constraint], general: &ThreadState, cache: &HeapCache) -> bool {
    for Constraint { a, t, k } in constraints {
        let t = eval_phi(t, phi);
        let k = eval_phi(k, phi);
        let a = eval_phi(a, phi);
        let v = index(&t, &k, general, cache);
        let cond = a.0.iter().any(|a| {
            a.subseteq(&v, &general.deref)
        });
        if !cond { return false }
    }

    true
}

fn index(t: &ValueSet, k: &ValueSet, st: &ThreadState, cache: &HeapCache) -> ValueSet {
    let mut vs = ValueSet::bottom();
    for t in &t.0 {
        for k in &k.0 {
            if let Some(v) = cache.get(&[t.clone(), k.clone()]) {
                vs = vs.union(v, &st.deref);
            }
        }
    }
    vs
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

type HeapCache = Map<[ValueParticle; 2], ValueSet>;

fn heap_cache(st: &ThreadState) -> HeapCache {
    let mut map = Map::new();
    for e in st.table_entries.iter() {
        let TableEntry::Add(t, k, v) = e else { continue };
        for t in &t.0 {
            let mut k_list: Vec<ValueParticle> = k.0.clone();
            while let Some(k) = k_list.pop() {
                let vv: &mut ValueSet = map.entry([t.clone(), k.clone()]).or_insert_with(|| ValueSet(Vec::new()));
                *vv = vv.union(v, &st.deref);
                if let Some(k_upc) = upcast(&k, &st.deref) {
                    k_list.extend(k_upc.0);
                }
            }
        }
    }

    map
}
