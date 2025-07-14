use crate::*;

pub struct ValueSet {
    symbols: Set<Symbol>,
    strings: OrTop<Set<String>>,
    ints: OrTop<Set<i64>>,
    tables: Set<TableSortId>,

    // you need to recursively deref these to evaluate the actual things in the ValueSet.
    value_ids: Set<ValueId>,
}

pub enum OrTop<T> {
    Top,
    T(T),
}
