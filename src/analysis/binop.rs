use crate::*;

pub fn step_binop(kind: BinOpKind, l: ValueParticle, r: ValueParticle, mut st: ThreadState) -> (ValueParticle, ThreadState) {
    use BinOpKind::*;
    use ValueParticle::*;

    let true_ = Symbol(crate::symbol::Symbol::new("True"));
    let false_ = Symbol(crate::symbol::Symbol::new("False"));
    let boolify = |b: bool| if b { true_.clone() } else { false_.clone() };
    let deref = &st.deref;

    let out = match (kind, l, r) {
        (Plus, Int(l), Int(r)) => Int(l + r),
        (Minus, Int(l), Int(r)) => Int(l - r),
        (Mul, Int(l), Int(r)) => Int(l * r),
        (Div, Int(l), Int(r)) => Int(l / r),
        (Mod, Int(l), Int(r)) => Int(l % r),
        (Pow, Int(l), Int(r)) => Int(l.pow(r as _)),
        (Lt, Int(l), Int(r)) => boolify(l < r),
        (Le, Int(l), Int(r)) => boolify(l <= r),
        (Gt, Int(l), Int(r)) => boolify(l > r),
        (Ge, Int(l), Int(r)) => boolify(l >= r),
        (kind, l, r) if l.overlaps(&TopInt, deref) && r.overlaps(&TopInt, deref) => {
            let vs = if matches!(kind, Lt|Le|Gt|Ge) {
                ValueSet(vec![true_, false_])
            } else {
                ValueSet(vec![TopInt])
            };
            let vid = crate::analysis::ValueId(crate::symbol::Symbol::new("binopVID"));
            st.deref.insert(vid, vs);
            ValueId(vid)
        },
        _ => unreachable!("return bottom here!"),
    };
    (out, st)
}

