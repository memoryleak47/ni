use crate::*;
use std::fmt::{self, Display, Formatter};

impl Display for IR {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (&pid, _) in self.procs.iter() {
            display_proc_header(pid, self, f)?;
            for st in &self.procs[&pid].stmts {
                display_stmt(st, f)?;
            }
            display_terminator(&self.procs[&pid].terminator, f)?;
            display_proc_footer(f)?;
        }

        Ok(())
    }
}

pub fn display_proc_header(pid: ProcId, ir: &IR, f: &mut Formatter<'_>) -> fmt::Result {
    let main_prefix = if ir.main_pid == pid { "main " } else { "" };

    write!(f, "{main_prefix}proc {pid} {{\n")
}

pub fn display_proc_footer(f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "}}\n\n")
}

fn display_stmt(
    stmt: &Statement,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    write!(f, "  ")?;

    use Statement::*;

    match stmt {
        Let(n, e, _) => {
            write!(f, "let {n} = ")?;
            display_expr(e, f)?;
        }
        Store(t, i, n) => {
            write!(f, "{t}[{i}] <- {n}")?;
        }
        Print(v) => write!(f, "print({})", v)?,
    }

    write!(f, ";\n")
}

fn display_terminator(
    terminator: &Terminator,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    write!(f, "  ")?;

    use Terminator::*;

    match terminator {
        Jmp(n) => write!(f, "jmp {}", n)?,
        Exit(n) => write!(f, "exit {}", n)?,
    }

    write!(f, ";\n")
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_stmt(self, f)
    }
}

impl Display for Terminator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_terminator(self, f)
    }
}

fn display_expr(expr: &Expr, f: &mut Formatter<'_>) -> fmt::Result {
    use Expr::*;
    match expr {
        Index(t, i) => write!( f, "{}[{}]", t, i)?,
        Root => write!(f, "@")?,
        NewTable => write!(f, "{{}}")?,
        Proc(pid) => write!(f, "{pid}")?,
        BinOp(kind, l, r) => write!(f, "{l} {kind} {r}")?,
        Index(l, r) => write!(f, "{l}[{r}]")?,
        Float(x) => write!(f, "{x}")?,
        Int(x) => write!(f, "{x}")?,
        Bool(true) => write!(f, "True")?,
        Bool(false) => write!(f, "False")?,
        Undef => write!(f, "Undef")?,
        Str(s) => write!(f, "\"{s}\"")?,
    }

    Ok(())
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_expr(self, f)
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

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", gsymb_get(*self))
    }
}
