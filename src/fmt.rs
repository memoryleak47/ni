use crate::*;
use std::fmt::{self, Display, Formatter};

const SIMPLIFY: bool = true;

impl Display for IR {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (&fid, _) in ordered_map_iter(self.fns.iter()) {
            display_fn_header(fid, self, f)?;
            let mut constmap = Map::new();
            for (&bid, _) in ordered_map_iter(self.fns[&fid].blocks.iter()) {
                display_block_header(fid, bid, self, f)?;
                for st in &self.fns[&fid].blocks[&bid] {
                    if SIMPLIFY
                        && let Statement::Compute(n, expr) = st
                        && let Some(x) = const_str(expr, &constmap)
                    {
                        constmap.insert(*n, x);
                    } else {
                        display_stmt(st, f, &constmap)?;
                    }
                }
            }
            display_fn_footer(f)?;
        }

        Ok(())
    }
}

fn const_str(e: &Expr, constmap: &Map<Node, String>) -> Option<String> {
    match e {
        Expr::BinOp(op, a, b) => {
            if let Some(a) = constmap.get(a) {
                if let Some(b) = constmap.get(b) {
                    return Some(format!("({a} {op} {b})"));
                }
            }
            None
        }
        Expr::Arg
        | Expr::Function(_)
        | Expr::Float(_)
        | Expr::Int(_)
        | Expr::Bool(_)
        | Expr::None
        | Expr::Str(_) => Some(e.to_string()),
        _ => None,
    }
}

pub fn display_fn_header(fid: FnId, ir: &IR, f: &mut Formatter<'_>) -> fmt::Result {
    let main_prefix = if ir.main_fn == fid { "main " } else { "" };

    write!(f, "{main_prefix}function f{fid}():\n")
}

pub fn display_fn_footer(f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "end\n\n")
}

pub fn display_block_header(
    fid: FnId,
    bid: BlockId,
    ir: &IR,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    write!(f, "  ")?;

    if bid == ir.fns[&fid].start_block {
        write!(f, "start block b{bid}:\n")?;
    } else {
        write!(f, "block b{bid}:\n")?;
    }

    Ok(())
}

fn display_stmt(
    stmt: &Statement,
    f: &mut Formatter<'_>,
    constmap: &Map<Node, String>,
) -> fmt::Result {
    write!(f, "    ")?;

    use Statement::*;

    match stmt {
        Compute(n, e) => {
            write!(f, "{} = ", node_string(*n, constmap))?;
            display_expr(e, f, constmap)?;
        }
        Store(t, i, n) => {
            write!(
                f,
                "{}[{}] <- {}",
                node_string(*t, constmap),
                node_string(*i, constmap),
                node_string(*n, constmap)
            )?;
        }
        If(cond, then_bid, else_bid) => {
            let cond = node_string(*cond, constmap);
            let then = block_id_string(*then_bid);
            let else_ = block_id_string(*else_bid);
            write!(f, "if {cond} then {then} else {else_}",)?;
        }
        FnCall(n, t) => write!(
            f,
            "{}({})",
            node_string(*n, constmap),
            node_string(*t, constmap)
        )?,
        Print(v) => write!(f, "print({})", node_string(*v, constmap))?,
        Throw(s) => write!(f, "throw('{s}')")?,
        Return => write!(f, "return")?,
    }

    write!(f, ";\n")
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_stmt(self, f, &Default::default())
    }
}

fn display_expr(expr: &Expr, f: &mut Formatter<'_>, constmap: &Map<Node, String>) -> fmt::Result {
    use Expr::*;
    match expr {
        Index(t, i) => write!(
            f,
            "{}[{}]",
            node_string(*t, constmap),
            node_string(*i, constmap)
        )?,
        Arg => write!(f, "arg")?,
        NewTable => write!(f, "{{}}")?,
        Function(fid) => write!(f, "{}", fn_id_string(*fid))?,
        BinOp(BinOpKind::Subscript, l, r) => write!(
            f,
            "{}[{}]",
            node_string(*l, constmap),
            node_string(*r, constmap)
        )?,
        BinOp(kind, l, r) => write!(
            f,
            "{} {} {}",
            node_string(*l, constmap),
            kind,
            node_string(*r, constmap)
        )?,
        Float(x) => write!(f, "{}", x)?,
        Int(x) => write!(f, "{}", x)?,
        Bool(true) => write!(f, "True")?,
        Bool(false) => write!(f, "False")?,
        None => write!(f, "None")?,
        Undef => write!(f, "Undef")?,
        Str(s) => write!(f, "\"{}\"", s)?,
    }

    Ok(())
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_expr(self, f, &Default::default())
    }
}

impl Display for BinOpKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use BinOpKind::*;
        let s = match self {
            Plus => "+",
            Minus => "-",
            Mul => "*",
            Div => "/",
            Mod => "%",
            Lt => "<",
            Le => "<=",
            Gt => ">",
            Ge => ">=",
            IsEqual => "==",
            IsNotEqual => "~=",
            Pow => "^",
            Subscript => "[_]",
        };
        write!(f, "{}", s)
    }
}

pub fn block_id_string(block_id: BlockId) -> String {
    format!("b{block_id}")
}
pub fn fn_id_string(fid: FnId) -> String {
    format!("f{fid}")
}
pub fn node_string(n: Node, constmap: &Map<Node, String>) -> String {
    constmap
        .get(&n)
        .cloned()
        .unwrap_or_else(|| format!("%{}", n))
}

fn ordered_map_iter<'s, K: Ord + 's, V: 's>(
    it: impl Iterator<Item = (&'s K, &'s V)>,
) -> impl Iterator<Item = (&'s K, &'s V)> {
    let mut v: Vec<_> = it.collect();
    v.sort_by_key(|x| x.0);
    v.into_iter()
}
