use crate::merger_analysis::*;
use std::fmt::{self, Debug, Formatter};

impl Debug for ValueParticle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueParticle::Symbol(s) => write!(f, "{s}"),
            ValueParticle::String(s) => write!(f, "\"{s}\""),
            ValueParticle::TopString => write!(f, "TopString"),
            ValueParticle::Int(i) => write!(f, "{i}"),
            ValueParticle::TopInt => write!(f, "TopInt"),
            ValueParticle::TableSort(TableSortId(ts)) => write!(f, "TS:{ts}"),
            ValueParticle::ValueId(ValueId(i)) => write!(f, "VID:{i}"),
            ValueParticle::Top => write!(f, "Top"),
        }
    }
}

impl Debug for ValueSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let n = self.0.len();
        for (i, x) in self.0.iter().enumerate() {
            write!(f, "{x:?}")?;
            if i != n-1 { write!(f, " | ")?; }
        }
        Ok(())
    }
}
