use crate::*;

#[derive(Clone)]
pub struct ValueSet(pub Vec<ValueParticle>); // disjunction of possibilities.

// So far, ValueParticles like Symbol(_), String(_) and Int(_) are not hashconsed into ValueIds!
#[derive(Clone)]
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
    fn subseteq(&self, other: &ValueSet) -> bool {
        todo!()
    }

    pub fn deref(&self, deref: &Deref) -> ValueSet {
        ValueSet(vec![self.clone()]).deref(deref)
    }
}

impl ValueSet {
    pub fn bottom() -> Self {
        Self(Vec::new())
    }

    pub fn compactify(self) -> Self {
        let mut v: Vec<ValueParticle> = self.0;
        for i in (0..v.len()).rev() {
            let t = v.swap_remove(i);
            let wv = Self(v);
            let b = !t.subseteq(&wv);
            v = wv.0;
            if b {
                v.push(t);
            }
        }
        Self(v)
    }

    pub fn union(&self, other: &ValueSet) -> ValueSet {
        let merge = Self(self.0.iter().chain(other.0.iter()).cloned().collect());
        merge.compactify()
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
        todo!()
    }

    pub fn intersection(&self, other: &ValueSet, deref: &Deref) -> Vec<ValueSet> {
        todo!()
    }
}
