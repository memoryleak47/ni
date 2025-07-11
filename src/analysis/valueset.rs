struct ValueSet {
    symbols: Set<Symbol>,
    strings: OrTop<Set<String>>,
    ints: OrTop<Set<Int>>,
    floats: OrTop<Set<Float>>,
    tables: Set<TableSortId>,

    // you need to recursively deref these to evaluate the actual things in the ValueSet.
    value_ids: Set<ValueId>,
}

enum OrTop<T> {
    Top,
    T(T),
}
