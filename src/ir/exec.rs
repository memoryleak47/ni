use crate::ir::*;

type TablePtr = usize;

fn table_get(ptr: TablePtr, idx: Value, ctxt: &mut Ctxt) -> Value {
    ctxt.heap[ptr]
        .entries
        .iter()
        .find(|(x, _)| *x == idx)
        .map(|(_, v)| v.clone())
        .unwrap_or(Value::Undef)
}

fn table_set(ptr: TablePtr, idx: Value, val: Value, ctxt: &mut Ctxt) {
    if idx == Value::Undef {
        crash("setting index with Undef is forbidden!", ctxt);
    }

    let data: &mut TableData = ctxt
        .heap
        .get_mut(ptr)
        .expect("table_set got dangling pointer!");
    data.entries.retain(|(x, _)| *x != idx);
    if val != Value::Undef {
        data.entries.push((idx.clone(), val));
    }
}

#[derive(Clone, PartialEq, Debug)]
enum Value {
    Undef,
    Bool(bool),
    TablePtr(TablePtr),
    Str(String),
    Float(R64),
    Int(i64),
    Symbol(Symbol),
}

#[derive(Debug)]
struct Ctxt<'ir> {
    heap: Vec<TableData>,
    root: Value,
    ir: &'ir IR,
    pid: Symbol,
    nodes: Map<Node, Value>,
    statement_idx: usize,
}

#[derive(Default, Debug)]
struct TableData {
    entries: Vec<(Value, Value)>,
}

fn exec_expr(expr: &Expr, ctxt: &mut Ctxt) -> Value {
    match expr {
        Expr::Index(t, idx) => {
            let t = ctxt.nodes[t].clone();
            let idx = ctxt.nodes[idx].clone();

            let Value::TablePtr(t) = t else {
                crash(&format!("indexing into non-table {:?}, with index {:?}!", t, idx), ctxt)
            };
            table_get(t, idx, ctxt)
        }
        Expr::Root => ctxt.root.clone(),
        Expr::NewTable => alloc_table(ctxt),
        Expr::BinOp(kind, l, r) => {
            let l = ctxt.nodes[l].clone();
            let r = ctxt.nodes[r].clone();

            exec_binop(kind.clone(), l, r, ctxt)
        }
        Expr::Symbol(s) => Value::Symbol(*s),
        Expr::Float(x) => Value::Float(*x),
        Expr::Int(x) => Value::Int(*x),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Undef => Value::Undef,
        Expr::Str(s) => Value::Str(s.clone()),
    }
}

fn exec_binop(kind: BinOpKind, l: Value, r: Value, ctxt: &mut Ctxt) -> Value {
    use BinOpKind::*;

    match (kind, l, r) {
        // int
        (Plus, Value::Int(l), Value::Int(r)) => Value::Int(l + r),
        (Minus, Value::Int(l), Value::Int(r)) => Value::Int(l - r),
        (Mul, Value::Int(l), Value::Int(r)) => Value::Int(l * r),
        (Div, Value::Int(l), Value::Int(r)) => Value::Int(l / r),
        (Mod, Value::Int(l), Value::Int(r)) => Value::Int(l % r),
        (Pow, Value::Int(l), Value::Int(r)) => Value::Int(l.pow(r as _)),
        (Lt, Value::Int(l), Value::Int(r)) => Value::Bool(l < r),
        (Le, Value::Int(l), Value::Int(r)) => Value::Bool(l <= r),
        (Gt, Value::Int(l), Value::Int(r)) => Value::Bool(l > r),
        (Ge, Value::Int(l), Value::Int(r)) => Value::Bool(l >= r),

        // float
        (Plus, Value::Float(l), Value::Float(r)) => Value::Float(l + r),
        (Minus, Value::Float(l), Value::Float(r)) => Value::Float(l - r),
        (Mul, Value::Float(l), Value::Float(r)) => Value::Float(l * r),
        (Div, Value::Float(l), Value::Float(r)) => Value::Float(l / r),
        (Mod, Value::Float(l), Value::Float(r)) => Value::Float(l % r),
        (Pow, Value::Float(l), Value::Float(r)) => Value::Float(l.powf(r)),
        (Lt, Value::Float(l), Value::Float(r)) => Value::Bool(l < r),
        (Le, Value::Float(l), Value::Float(r)) => Value::Bool(l <= r),
        (Gt, Value::Float(l), Value::Float(r)) => Value::Bool(l > r),
        (Ge, Value::Float(l), Value::Float(r)) => Value::Bool(l >= r),

        (Plus, Value::Str(l), Value::Str(r)) => Value::Str(format!("{}{}", l, r)),

        (IsEqual, l, r) => Value::Bool(l == r),
        (IsNotEqual, l, r) => Value::Bool(l != r),
        (kind, l, r) => crash(&format!("type error! \"{l:?} {kind} {r:?}\""), ctxt),
    }
}

fn alloc_table(ctxt: &mut Ctxt) -> Value {
    let tid = ctxt.heap.len();
    ctxt.heap.push(Default::default());

    Value::TablePtr(tid)
}

pub fn exec(ir: &IR) {
    let mut ctxt = Ctxt {
        ir,
        heap: Vec::new(),
        root: Value::Undef,
        pid: ir.main_pid,
        nodes: Default::default(),
        statement_idx: 0,
    };
    let root_table = alloc_table(&mut ctxt);
    ctxt.root = root_table;

    while step(&mut ctxt) {}
}

fn step_stmt(stmt: &Statement, ctxt: &mut Ctxt) {
    ctxt.statement_idx += 1;

    use Statement::*;
    match stmt {
        Let(n, expr, _) => {
            let val = exec_expr(expr, ctxt);
            ctxt.nodes.insert(*n, val);
        }
        Store(t, idx, n) => {
            let t = ctxt.nodes[t].clone();
            let idx = ctxt.nodes[idx].clone();
            let val = ctxt.nodes[n].clone();
            let Value::TablePtr(t) = t.clone() else {
                crash("indexing into non-table!", ctxt)
            };
            table_set(t, idx, val, ctxt);
        }
        Print(n) => {
            let val = &ctxt.nodes[n];
            match val {
                Value::Undef => crash("print called on Undef!", ctxt),
                Value::Bool(true) => println!("True"),
                Value::Bool(false) => println!("False"),
                Value::Symbol(s) => println!("{s}"),
                Value::Str(s) => println!("{}", s),
                Value::TablePtr(ptr) => println!("table: {}", ptr),
                Value::Float(x) => println!("{}", x),
                Value::Int(x) => println!("{}", x),
            }
        }
    }
}

fn step_terminator(terminator: &Terminator, ctxt: &mut Ctxt) -> bool {
    use Terminator::*;
    match terminator {
        Jmp(n) => {
            match ctxt.nodes[n].clone() {
                Value::Symbol(pid) => {
                    ctxt.pid = pid;
                    ctxt.nodes.clear();
                    ctxt.statement_idx = 0;
                }
                v => crash(&format!("trying to execute non-function value! {:?}", v), ctxt),
            };
            true
        }
        Exit => false,
        Panic(n) => {
            let v = ctxt.nodes[n].clone();
            println!("PANIC: {v:?}");
            false
        }
    }
}

// returns "false" when done.
fn step(ctxt: &mut Ctxt) -> bool {
    let proc = &ctxt.ir.procs[&ctxt.pid];
    match proc.stmts.get(ctxt.statement_idx) {
        Some(stmt) => { step_stmt(stmt, ctxt); true }
        None => step_terminator(&proc.terminator, ctxt)
    }
}

fn crash(s: &str, ctxt: &Ctxt) -> ! {
    // let stmt = stringify_last_stmt(ctxt);
    println!("exec IR crashing due to '{s}' at stmt ???");
    std::process::exit(1);
}
