use crate::*;

#[derive(Clone)]
pub struct ValueSet(pub Vec<ValueParticle>); // disjunction of possibilities.

// So far, ValueParticles like Symbol(_), String(_) and Int(_) are not hashconsed into ValueIds!
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ValueParticle {
    Top,
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
        // We deref away all ValueIds.
        let a = self.deref(deref);
        let b = other.deref(deref);
        for a in &a.0 {
            for b in &b.0 {
                if a.overlaps_dereffed(b) { return true; }
            }
        }
        false
    }

    fn overlaps_dereffed(&self, other: &ValueParticle) -> bool {
        match (self, other) {
            (ValueParticle::Top, _) | (_, ValueParticle::Top) => true,
            (ValueParticle::String(_), ValueParticle::TopString) => true,
            (ValueParticle::TopString, ValueParticle::String(_)) => true,
            (ValueParticle::Int(_), ValueParticle::TopInt) => true,
            (ValueParticle::TopInt, ValueParticle::Int(_)) => true,
            (x, y) => x == y,
        }
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

    pub fn is_bottom(&self) -> bool { self.0.is_empty() }

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
        ValueSet(out).compactify(&Default::default())
    }

    pub fn subseteq(&self, other: &ValueSet, deref: &Deref) -> bool {
        self.0.iter().all(|x| x.subseteq(other, deref))
    }

    pub fn overlaps(&self, other: &ValueSet, deref: &Deref) -> bool {
        for x in &*self.0 {
            for y in &*other.0 {
                if x.overlaps(y, deref) { return true; }
            }
        }
        false
    }

    pub fn is_concrete(&self) -> bool {
        match &*self.0 {
            [x] => x.is_concrete(),
            _ => false,
        }
    }
}

pub fn upcast(p: &ValueParticle, deref: &Deref) -> Option<ValueSet> {
    match p {
        ValueParticle::ValueId(v) => deref.get(v).cloned(),
        ValueParticle::String(_) => Some(ValueSet(vec![ValueParticle::TopString])),
        ValueParticle::Int(_) => Some(ValueSet(vec![ValueParticle::TopInt])),
        ValueParticle::Top => None,
        _ => Some(ValueSet(vec![ValueParticle::Top])),
    }
}

