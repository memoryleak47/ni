use crate::*;

pub struct ValueSet {
    pub symbols: Set<Symbol>,
    pub strings: OrTop<Set<String>>,
    pub ints: OrTop<Set<i64>>,
    pub table_sorts: Set<TableSortId>,

    // you need to recursively deref these to evaluate the actual things in the ValueSet.
    pub value_ids: Set<ValueId>,
}

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
