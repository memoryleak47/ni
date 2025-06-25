use std::collections::BTreeMap;
use std::sync::*;

// global symbol map.
static GSYMB: LazyLock<Mutex<SymbolMap>> = LazyLock::new(|| Mutex::from(SymbolMap::new()));

fn gsymb_add(x: String) -> Symbol {
    let mut g = GSYMB.lock().unwrap();
    g.add(x)
}

fn gsymb_get(x: Symbol) -> String {
    let g = GSYMB.lock().unwrap();
    g.get(x).to_string()
}

fn gsymb_fresh() -> Symbol {
    let mut g = GSYMB.lock().unwrap();
    g.fresh()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Symbol(pub usize);

// implementation of symbol map.

struct SymbolMap {
    string_to_id: BTreeMap<String, Symbol>,
    id_to_string: Vec<String>,
    fresh_counter: usize,
}

impl SymbolMap {
    fn new() -> Self {
        Self {
            string_to_id: Default::default(),
            id_to_string: Default::default(),
            fresh_counter: 0,
        }
    }

    fn add(&mut self, x: String) -> Symbol {
        if let Some(y) = self.string_to_id.get(&x) {
            return *y;
        } else {
            let i = self.string_to_id.len();
            self.string_to_id.insert(x.clone(), Symbol(i));
            self.id_to_string.push(x);
            Symbol(i)
        }
    }

    fn get(&self, id: Symbol) -> &str {
        self.id_to_string.get(id.0).unwrap()
    }

    fn fresh(&mut self) -> Symbol {
        loop {
            let s = format!("_{}", self.fresh_counter);
            self.fresh_counter += 1;

            if self.string_to_id.get(&s).is_none() {
                return self.add(s);
            }
        }
    }
}


use std::cmp::*;

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Symbol) -> Option<Ordering> {
        let a = self;
        let b = other;

        let a = gsymb_get(*a);
        let b = gsymb_get(*b);

        for (ca, cb) in a.chars().zip(b.chars()) {
            let o = ca.cmp(&cb);
            if o != Ordering::Equal { return Some(o); }
        }

        let o = a.len().cmp(&b.len());
        Some(o)
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Symbol) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Symbol {
    pub fn new(s: String) -> Symbol {
        gsymb_add(s)
    }

    pub fn fresh() -> Symbol {
        gsymb_fresh()
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", gsymb_get(*self))
    }
}

