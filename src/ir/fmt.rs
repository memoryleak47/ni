use crate::*;
use std::fmt::{self, Display, Formatter};

impl Display for IR {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (&pid, _) in self.procs.iter() {
            display_proc(pid, self, f)?;
        }

        Ok(())
    }
}

pub fn display_proc(pid: Symbol, ir: &IR, f: &mut Formatter<'_>) -> fmt::Result {
    let main_prefix = if ir.main_pid == pid { "main " } else { "" };

    write!(f, "{main_prefix}proc {pid} {{\n")?;

    let proc = &ir.procs[&pid];
    for i in 0..proc.stmts.len() {
        display_stmt(i, proc, false, f)?;
    }
    write!(f, "}}\n\n")
}

fn display_stmt(stmt_id: usize, proc: &Proc, force_visible: bool, f: &mut Formatter<'_>) -> fmt::Result {
    use Statement::*;

    let stmt = &proc.stmts[stmt_id];
    match stmt {
        Let(n, e, visible) => {
            let e_str = expr_string(e, proc);
            if *visible || force_visible {
                write!(f, "    {n} = {e_str};\n")?;
            }
        }
        Store(t, i, n) => {
            let idx = display_index(*t, *i, proc);
            let n = node_string(*n, proc);
            write!(f, "    {idx} = {n};\n")?;
        }
        Print(v) => {
            let v = node_string(*v, proc);
            write!(f, "    print {v};\n")?;
        },
        Jmp(n) => {
            let n = node_string(*n, proc);
            write!(f, "    jmp {n};\n")?;
        },
        Exit => write!(f, "    exit;\n")?,
        Panic(n) => {
            let n = node_string(*n, proc);
            write!(f, "    panic {n};\n")?;
        },
    }

    Ok(())
}

fn expr_string(expr: &Expr, proc: &Proc) -> String {
    use Expr::*;
    match expr {
        Index(t, i) => display_index(*t, *i, proc),
        Root => format!("@"),
        NewTable => format!("{{}}"),
        BinOp(kind, l, r) => {
            let l = node_string(*l, proc);
            let r = node_string(*r, proc);
            format!("{l} {kind} {r}")
        }
        Index(l, r) => {
            let l = node_string(*l, proc);
            let r = node_string(*r, proc);
            format!("{l}[{r}]")
        }
        Symbol(s) => format!("{s}"),
        Float(x) => format!("{x}"),
        Int(x) => format!("{x}"),
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

fn node_string(n: Node, proc: &Proc) -> String {
    let (expr, b) = get_def(n, proc);
    if !b {
        expr_string(expr, proc)
    } else {
        format!("{n}")
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

fn get_def(n: Node, proc: &Proc) -> (&Expr, bool) {
    for x in proc.stmts.iter() {
        if let Statement::Let(n2, expr, b) = x && n == *n2 {
            return (expr, *b);
        }
    }
    panic!("Can't find definition for node {n}")
}

fn display_index(t: Node, i: Node, proc: &Proc) -> String {
    let (e, _) = get_def(i, proc);
    let b = matches!(e, Expr::Symbol(_));
    let t = node_string(t, proc);
    let i = node_string(i, proc);

    if b {
        format!("{t}.{i}")
    } else {
        format!("{t}[{i}]")
    }
}

pub struct StmtFmt<'a> {
    pub stmt_id: usize,
    pub proc: &'a Proc,
}

impl<'a> Display for StmtFmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_stmt(self.stmt_id, self.proc, true, f)
    }
}
