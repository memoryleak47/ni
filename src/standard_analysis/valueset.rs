use crate::standard_analysis::*;

#[derive(Clone, Default)]
pub struct ValueSet(pub Vec<ValueParticle>); // disjunction of possibilities.

pub type Location = (Symbol, usize);

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ValueParticle {
    Symbol(Symbol),
    String(String),
    TopString,
    Int(i64),
    TopInt,
    Summary(Location),
    Concrete(Location),
}

impl ValueParticle {
    pub fn subseteq(&self, other: &ValueSet) -> bool {
        if other.0.contains(self) { return true; }
        if matches!(self, ValueParticle::String(_)) && other.0.contains(&ValueParticle::TopString) { return true; }
        if matches!(self, ValueParticle::Int(_)) && other.0.contains(&ValueParticle::TopInt) { return true; }
        false
    }

    pub fn overlaps(&self, other: &ValueParticle) -> bool {
        match (self, other) {
            (ValueParticle::String(_), ValueParticle::TopString) => true,
            (ValueParticle::TopString, ValueParticle::String(_)) => true,
            (ValueParticle::Int(_), ValueParticle::TopInt) => true,
            (ValueParticle::TopInt, ValueParticle::Int(_)) => true,
            (x, y) => x == y,
        }
    }

    pub fn is_concrete(&self) -> bool {
        matches!(self, ValueParticle::Concrete(_)
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

    pub fn compactify(mut self) -> Self {
        let top_str = self.0.contains(&ValueParticle::TopString);
        let top_int = self.0.contains(&ValueParticle::TopInt);
        if top_str { self.0.retain(|x| !matches!(x, ValueParticle::String(_))); }
        if top_int { self.0.retain(|x| !matches!(x, ValueParticle::Int(_))); }
        self
    }

    pub fn union(&self, other: &ValueSet) -> ValueSet {
        let merge = Self(self.0.iter().chain(other.0.iter()).cloned().collect());
        merge.compactify()
    }

    pub fn subseteq(&self, other: &ValueSet) -> bool {
        self.0.iter().all(|x| x.subseteq(other))
    }

    pub fn overlaps(&self, other: &ValueSet) -> bool {
        for x in &*self.0 {
            for y in &*other.0 {
                if x.overlaps(y) { return true; }
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

impl PartialEq for ValueSet {
    fn eq(&self, other: &ValueSet) -> bool {
        self.subseteq(other) && other.subseteq(self)
    }
}

impl Eq for ValueSet {}
