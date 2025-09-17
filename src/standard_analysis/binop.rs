use crate::standard_analysis::*;

pub fn step_binop(kind: BinOpKind, l: ValueSet, r: ValueSet, mut st: ProcState) -> (ValueSet, ProcState) {
    use BinOpKind::*;
    use ValueParticle::*;

    let true_ = Symbol(crate::symbol::Symbol::new("True"));
    let false_ = Symbol(crate::symbol::Symbol::new("False"));
    let boolify = |b: bool| if b { true_.clone() } else { false_.clone() };

    let mut vs = ValueSet::bottom();
    for l in &l.0 {
        for r in &r.0 {
            let out = match (kind, l, r) {
                (Plus, Int(l), Int(r)) => vs.0.push(Int(l + r)),
                (Minus, Int(l), Int(r)) => vs.0.push(Int(l - r)),
                (Mul, Int(l), Int(r)) => vs.0.push(Int(l * r)),
                (Div, Int(l), Int(r)) => vs.0.push(Int(l / r)),
                (Mod, Int(l), Int(r)) => vs.0.push(Int(l % r)),
                (Pow, Int(l), Int(r)) => vs.0.push(Int(l.pow(*r as _))),
                (Lt, Int(l), Int(r)) => vs.0.push(boolify(l < r)),
                (Le, Int(l), Int(r)) => vs.0.push(boolify(l <= r)),
                (Gt, Int(l), Int(r)) => vs.0.push(boolify(l > r)),
                (Ge, Int(l), Int(r)) => vs.0.push(boolify(l >= r)),
                (kind, l, r) if l.overlaps(&TopInt) && r.overlaps(&TopInt) => {
                    let vs = if matches!(kind, Lt|Le|Gt|Ge) {
                        vs.0.extend([true_.clone(), false_.clone()]);
                    } else {
                        vs.0.push(TopInt);
                    };
                },
                _ => {},
            };
        }
    }
    (vs.compactify(), st)
}

