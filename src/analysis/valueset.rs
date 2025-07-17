use crate::*;

#[derive(Clone, Debug)]
pub struct ValueSet(pub Vec<ValueParticle>); // disjunction of possibilities.

// So far, ValueParticles like Symbol(_), String(_) and Int(_) are not hashconsed into ValueIds!
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ValueParticle {
    Symbol(Symbol),
    String(String),
    TopString,
    Int(i64),
    TopInt,
    TableSort(TableSortId),
    ValueId(ValueId),
}

impl ValueParticle {
    pub fn subseteq(&self, other: &ValueSet, deref: &Deref) -> bool {
        if other.0.contains(self) { return true; }
        if let Some(x) = upcast(self, deref) {
            return x.subseteq(other, deref);
        }
        false
    }

    pub fn deref(&self, deref: &Deref) -> ValueSet {
        ValueSet(vec![self.clone()]).deref(deref)
    }

    pub fn overlaps(&self, other: &ValueParticle, deref: &Deref) -> bool {
        // NOTE: this only works as `intersect_p` is overapproximating instead of underapproximating when comparing a ValueId to something else.
        intersect_p(self, other, deref).0.len() > 0
    }

    pub fn is_concrete(&self) -> bool {
        matches!(self, ValueParticle::ValueId(_)
            | ValueParticle::String(_)
            | ValueParticle::Int(_)
            | ValueParticle::Symbol(_)
        )
    }
}

impl ValueSet {
    pub fn bottom() -> Self {
        Self(Vec::new())
    }

    pub fn compactify(self, deref: &Deref) -> Self {
        let mut v: Vec<ValueParticle> = self.0;
        for i in (0..v.len()).rev() {
            let t = v.swap_remove(i);
            let wv = Self(v);
            let b = !t.subseteq(&wv, deref);
            v = wv.0;
            if b {
                v.push(t);
            }
        }
        Self(v)
    }

    pub fn union(&self, other: &ValueSet, deref: &Deref) -> ValueSet {
        let merge = Self(self.0.iter().chain(other.0.iter()).cloned().collect());
        merge.compactify(deref)
    }

    pub fn deref(&self, deref: &Deref) -> ValueSet {
        let mut out = Vec::new();
        for x in self.0.iter() {
            match x {
                ValueParticle::ValueId(i) => out.extend(deref[i].deref(deref).0),
                _ => out.push(x.clone()),
            }
        }
        ValueSet(out)
    }

    pub fn subseteq(&self, other: &ValueSet, deref: &Deref) -> bool {
        self.0.iter().all(|x| x.subseteq(other, deref))
    }

    pub fn is_concrete(&self) -> bool {
        match &*self.0 {
            [x] => x.is_concrete(),
            _ => false,
        }
    }
}
