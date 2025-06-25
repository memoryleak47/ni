use crate::*;
use std::fmt::{self, Display, Formatter};

const SHOW_ALL: bool = false;

impl Display for IR {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (&pid, _) in self.procs.iter() {
            display_proc(pid, self, f)?;
        }

        Ok(())
    }
}

pub fn display_proc(pid: ProcId, ir: &IR, f: &mut Formatter<'_>) -> fmt::Result {
    let main_prefix = if ir.main_pid == pid { "main " } else { "" };

    write!(f, "{main_prefix}proc {pid} {{\n")?;

    let proc = &ir.procs[&pid];
    let mut nodemap = Map::new();
    for st in &proc.stmts {
        display_stmt(st, &mut nodemap, f)?;
    }
    display_terminator(&ir.procs[&pid].terminator, &nodemap, f)?;
    write!(f, "}}\n\n")
}

fn display_stmt(stmt: &Statement, nodemap: &mut Map<Node, String>, f: &mut Formatter<'_>,) -> fmt::Result {
    use Statement::*;

    match stmt {
        Let(n, e, visible) => {
            let e = expr_string(e, nodemap);
            if *visible || SHOW_ALL {
                write!(f, "    let {n} = {e};\n")?;
            } else {
                nodemap.insert(*n, e);
            }
        }
        Store(t, i, n) => {
            let t = node_string(*t, nodemap);
            let i = node_string(*i, nodemap);
            let n = node_string(*n, nodemap);
            write!(f, "    {t}[{i}] <- {n};\n")?;
        }
        Print(v) => write!(f, "    print {};\n", v)?,
    }

    Ok(())
}

fn display_terminator(terminator: &Terminator, nodemap: &Map<Node, String>, f: &mut Formatter<'_>) -> fmt::Result {
    use Terminator::*;

    match terminator {
        Jmp(n) => {
            let n = node_string(*n, nodemap);
            write!(f, "    jmp {n};\n")?;
        },
        Exit(n) => {
            let n = node_string(*n, nodemap);
            write!(f, "    exit {n};\n")?;
        },
    }

    Ok(())
}

fn expr_string(expr: &Expr, nodemap: &Map<Node, String>) -> String {
    use Expr::*;
    match expr {
        Index(t, i) => {
            let t = node_string(*t, nodemap);
            let i = node_string(*i, nodemap);
            format!("{t}[{i}]")
        },
        Root => format!("@"),
        NewTable => format!("{{}}"),
        Proc(pid) => format!("{pid}"),
        BinOp(kind, l, r) => {
            let l = node_string(*l, nodemap);
            let r = node_string(*r, nodemap);
            format!("{l} {kind} {r}")
        }
        Index(l, r) => {
            let l = node_string(*l, nodemap);
            let r = node_string(*r, nodemap);
            format!("{l}[{r}]")
        }
        Symbol(s) => format!("${s}"),
        Float(x) => format!("{x}"),
        Int(x) => format!("{x}"),
        Bool(true) => format!("True"),
        Bool(false) => format!("False"),
        Undef => format!("Undef"),
        Str(s) => format!("\"{s}\""),
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
        };
        write!(f, "{}", s)
    }
}

impl Display for ProcId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn node_string(n: Node, nodemap: &Map<Node, String>) -> String {
    match nodemap.get(&n) {
        Some(x) => x.to_string(),
        None => n.to_string(),
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}
