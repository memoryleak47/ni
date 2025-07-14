use crate::*;

#[derive(Clone)]
pub struct ValueSet {
    pub symbols: Set<Symbol>,
    pub strings: OrTop<Set<String>>,
    pub ints: OrTop<Set<i64>>,
    pub table_sorts: Set<TableSortId>,

    // you need to recursively deref these to evaluate the actual things in the ValueSet.
    pub value_ids: Set<ValueId>,
}

#[derive(Clone)]
pub enum OrTop<T> {
    Top,
    T(T),
}

impl ValueSet {
    pub fn bottom() -> Self {
        ValueSet {
            symbols: Set::new(),
            strings: OrTop::T(Set::new()),
            ints: OrTop::T(Set::new()),
            table_sorts: Set::new(),
            value_ids: Set::new(),
        }
    }
}

impl<T> OrTop<Set<T>> where T: Hash + Eq + Clone {
    pub fn insert(&mut self, t: T) {
        match self {
            OrTop::Top => {},
            OrTop::T(s) => { s.insert(t); },
        }
    }

    pub fn union(&self, other: &OrTop<Set<T>>) -> OrTop<Set<T>> {
        match (self, other) {
            (OrTop::Top, _) => OrTop::Top,
            (_, OrTop::Top) => OrTop::Top,
            (OrTop::T(s1), OrTop::T(s2)) => OrTop::T(s1.union(&s2).cloned().collect()),
        }
    }

    pub fn overlaps(&self, other: &OrTop<Set<T>>) -> bool {
        match (self, other) {
            (OrTop::Top, OrTop::Top) => true,
            (OrTop::Top, OrTop::T(a)) => a.len() > 0,
            (OrTop::T(a), OrTop::Top) => a.len() > 0,
            (OrTop::T(s1), OrTop::T(s2)) => s1.intersection(s2).next().is_some(),
        }
    }
}

impl ValueSet {
    pub fn union(&self, other: &ValueSet) -> ValueSet {
        ValueSet {
            symbols: self.symbols.union(&other.symbols).cloned().collect(),
            strings: self.strings.union(&other.strings),
            ints: self.ints.union(&other.ints),
            table_sorts: self.table_sorts.union(&other.table_sorts).cloned().collect(),
            value_ids: self.value_ids.union(&other.value_ids).cloned().collect(),
        }
    }

    pub fn overlaps(&self, other: &ValueSet) -> bool {
        self.symbols.intersection(&other.symbols).next().is_some() ||
        self.strings.overlaps(&other.strings) ||
        self.ints.overlaps(&other.ints) ||
        self.table_sorts.intersection(&other.table_sorts).next().is_some()
        // XXX TODO: cover value ids
    }
}

pub fn full_deref_vs(mut v: ValueSet, st: &ThreadState) -> ValueSet {
    let deref = &st.deref_val_id;
    while v.value_ids.len() > 0 {
        let mut value_ids = Set::new();
        std::mem::swap(&mut value_ids, &mut v.value_ids);
        for x in value_ids {
            v = v.union(&deref[&x]);
        }
    }
    v
}

pub fn full_deref(vid: ValueId, st: &ThreadState) -> ValueSet {
    full_deref_vs(st.deref_val_id[&vid].clone(), st)
}
