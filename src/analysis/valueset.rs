use crate::*;

#[derive(Clone)]
pub struct ValueSet {
    pub top: bool, // true means this ValueSet contains all values.
    pub symbols: Set<Symbol>,
    pub strings: OrTop<Set<String>>,
    pub ints: OrTop<Set<i64>>,
    pub table_sorts: Set<TableSortId>,

    // you need to recursively deref these to evaluate the actual things in the ValueSet.
    // So, this `value_ids` set is a disjunction.
    // The value is equal to either one of these ValueIds, or one of symbols, strings, ints, table_sorts.
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
            top: false,
            symbols: Set::new(),
            strings: OrTop::T(Set::new()),
            ints: OrTop::T(Set::new()),
            table_sorts: Set::new(),
            value_ids: Set::new(),
        }
    }

    pub fn top() -> Self {
        ValueSet {
            top: true,
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

    pub fn intersection(&self, other: &OrTop<Set<T>>) -> OrTop<Set<T>> {
        match (self, other) {
            (OrTop::Top, x) => x.clone(),
            (x, OrTop::Top) => x.clone(),
            (OrTop::T(s1), OrTop::T(s2)) => OrTop::T(s1.intersection(&s2).cloned().collect()),
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

    pub fn is_empty(&self) -> bool {
        match self {
            OrTop::Top => false,
            OrTop::T(a) => a.is_empty(),
        }
    }

    pub fn is_subset(&self, other: &OrTop<Set<T>>) -> bool {
        match (self, other) {
            (_, OrTop::Top) => true,
            (OrTop::Top, OrTop::T(a)) => false,
            (OrTop::T(s1), OrTop::T(s2)) => s1.is_subset(s2),
        }
    }
}

impl ValueSet {
    pub fn union(&self, other: &ValueSet) -> ValueSet {
        if self.top { return self.clone(); }
        if other.top { return other.clone(); }

        ValueSet {
            top: false,
            symbols: self.symbols.union(&other.symbols).cloned().collect(),
            strings: self.strings.union(&other.strings),
            ints: self.ints.union(&other.ints),
            table_sorts: self.table_sorts.union(&other.table_sorts).cloned().collect(),
            value_ids: self.value_ids.union(&other.value_ids).cloned().collect(),
        }
    }

    pub fn overlaps(&self, other: &ValueSet, d: &Deref) -> bool {
        let l = full_deref_vs(self.clone(), d);
        let r = full_deref_vs(other.clone(), d);

        if l.is_bot() || r.is_bot() { return false; }
        if l.top || r.top { return true; }

        l.symbols.intersection(&r.symbols).next().is_some() ||
        l.strings.overlaps(&r.strings) ||
        l.ints.overlaps(&r.ints) ||
        l.table_sorts.intersection(&r.table_sorts).next().is_some()
    }

    pub fn is_bot(&self) -> bool {
        !self.top
        && self.symbols.is_empty()
        && self.strings.is_empty()
        && self.ints.is_empty()
        && self.table_sorts.is_empty()
        && self.value_ids.is_empty()
    }

    // TODO respect ValueIds.
    pub fn is_subset(&self, other: &ValueSet, st: &ThreadState) -> bool {
        if other.top { return true; }
        if self.top { return false; }

        self.symbols.is_subset(&other.symbols)
        && self.strings.is_subset(&other.strings)
        && self.ints.is_subset(&other.ints)
        && self.table_sorts.is_subset(&other.table_sorts)
        && self.value_ids.is_subset(&other.value_ids)
    }

    // TODO respect ValueIds.
    pub fn intersection(&self, other: &ValueSet, st: &ThreadState) -> ValueSet {
        if self.top { return other.clone(); }
        if other.top { return self.clone(); }

        ValueSet {
            top: false,
            symbols: self.symbols.intersection(&other.symbols).cloned().collect(),
            strings: self.strings.intersection(&other.strings),
            ints: self.ints.intersection(&other.ints),
            table_sorts: self.table_sorts.intersection(&other.table_sorts).cloned().collect(),
            value_ids: self.value_ids.intersection(&other.value_ids).cloned().collect(),
        }
    }

    pub fn concrete_eq(&self, other: &ValueSet) -> bool {
        if self.top || other.top { return false; }

        todo!()
    }
}

pub fn full_deref_vs(mut v: ValueSet, deref: &Deref) -> ValueSet {
    while v.value_ids.len() > 0 {
        let mut value_ids = Set::new();
        std::mem::swap(&mut value_ids, &mut v.value_ids);
        for x in value_ids {
            v = v.union(&deref[&x]);
        }
    }
    v
}

pub fn full_deref(vid: ValueId, deref: &Deref) -> ValueSet {
    full_deref_vs(deref[&vid].clone(), deref)
}
