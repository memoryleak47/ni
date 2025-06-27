use crate::ir::*;

type TablePtr = usize;

fn table_get(ptr: TablePtr, idx: Value, ctxt: &mut Ctxt) -> Value {
    ctxt.heap[ptr]
        .entries
        .iter()
        .find(|(x, _)| *x == idx)
        .map(|(_, v)| v.clone())
        .unwrap_or(ctxt.undef_v.clone())
}

fn table_set(ptr: TablePtr, idx: Value, val: Value, ctxt: &mut Ctxt) {
    if idx == ctxt.undef_v {
        crash("setting index with Undef is forbidden!", ctxt);
    }

    let data: &mut TableData = ctxt
        .heap
        .get_mut(ptr)
        .expect("table_set got dangling pointer!");
    data.entries.retain(|(x, _)| *x != idx);
    if val != ctxt.undef_v {
        data.entries.push((idx.clone(), val));
    }
}

#[derive(Clone, PartialEq, Debug)]
enum Value {
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
    last_stmt: String,

    true_v: Value,
    false_v: Value,
    undef_v: Value,
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
        Expr::Str(s) => Value::Str(s.clone()),
    }
}

fn exec_binop(kind: BinOpKind, l: Value, r: Value, ctxt: &mut Ctxt) -> Value {
    use BinOpKind::*;

    let boolify = |b: bool| -> Value {
        if b { ctxt.true_v.clone() }
        else { ctxt.false_v.clone() }
    };

    match (kind, l, r) {
        // int
        (Plus, Value::Int(l), Value::Int(r)) => Value::Int(l + r),
        (Minus, Value::Int(l), Value::Int(r)) => Value::Int(l - r),
        (Mul, Value::Int(l), Value::Int(r)) => Value::Int(l * r),
        (Div, Value::Int(l), Value::Int(r)) => Value::Int(l / r),
        (Mod, Value::Int(l), Value::Int(r)) => Value::Int(l % r),
        (Pow, Value::Int(l), Value::Int(r)) => Value::Int(l.pow(r as _)),
        (Lt, Value::Int(l), Value::Int(r)) => boolify(l < r),
        (Le, Value::Int(l), Value::Int(r)) => boolify(l <= r),
        (Gt, Value::Int(l), Value::Int(r)) => boolify(l > r),
        (Ge, Value::Int(l), Value::Int(r)) => boolify(l >= r),

        // float
        (Plus, Value::Float(l), Value::Float(r)) => Value::Float(l + r),
        (Minus, Value::Float(l), Value::Float(r)) => Value::Float(l - r),
        (Mul, Value::Float(l), Value::Float(r)) => Value::Float(l * r),
        (Div, Value::Float(l), Value::Float(r)) => Value::Float(l / r),
        (Mod, Value::Float(l), Value::Float(r)) => Value::Float(l % r),
        (Pow, Value::Float(l), Value::Float(r)) => Value::Float(l.powf(r)),
        (Lt, Value::Float(l), Value::Float(r)) => boolify(l < r),
        (Le, Value::Float(l), Value::Float(r)) => boolify(l <= r),
        (Gt, Value::Float(l), Value::Float(r)) => boolify(l > r),
        (Ge, Value::Float(l), Value::Float(r)) => boolify(l >= r),

        (Plus, Value::Str(l), Value::Str(r)) => Value::Str(format!("{}{}", l, r)),

        (IsEqual, l, r) => boolify(l == r),
        (IsNotEqual, l, r) => boolify(l != r),
        (kind, l, r) => crash(&format!("type error! \"{l:?} {kind} {r:?}\""), ctxt),
    }
}

fn alloc_table(ctxt: &mut Ctxt) -> Value {
    let tid = ctxt.heap.len();
    ctxt.heap.push(Default::default());

    Value::TablePtr(tid)
}

pub fn exec(ir: &IR) {
    let undef_v = Value::Symbol(Symbol::new("Undef"));
    let true_v = Value::Symbol(Symbol::new("True"));
    let false_v = Value::Symbol(Symbol::new("False"));
    let mut ctxt = Ctxt {
        ir,
        heap: Vec::new(),
        root: undef_v.clone(),
        pid: ir.main_pid,
        nodes: Default::default(),
        statement_idx: 0,
        last_stmt: "<none>".to_string(),
        undef_v,
        true_v,
        false_v,
    };
    let root_table = alloc_table(&mut ctxt);
    ctxt.root = root_table;

    while step(&mut ctxt) {}
}

fn step_stmt(stmt: &Statement, ctxt: &mut Ctxt) -> bool {
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
                Value::Symbol(s) => println!("{s}"),
                Value::Str(s) => println!("{}", s),
                Value::TablePtr(ptr) => println!("table: {}", ptr),
                Value::Float(x) => println!("{}", x),
                Value::Int(x) => println!("{}", x),
            }
        }
        Jmp(n) => {
            match ctxt.nodes[n].clone() {
                Value::Symbol(pid) => {
                    ctxt.pid = pid;
                    ctxt.nodes.clear();
                    ctxt.statement_idx = 0;
                }
                v => crash(&format!("trying to execute non-function value! {:?}", v), ctxt),
            };
        }
        Exit => return false,
        Panic(n) => {
            let v = ctxt.nodes[n].clone();
            println!("PANIC: {v:?}");
            return false;
        }
    }

    true
}

// returns "false" when done.
fn step(ctxt: &mut Ctxt) -> bool {
    let proc = &ctxt.ir.procs[&ctxt.pid];
    let stmt = &proc.stmts.get(ctxt.statement_idx).unwrap();
    ctxt.last_stmt = StmtFmt { stmt_id: ctxt.statement_idx, proc }.to_string();

    step_stmt(stmt, ctxt)
}

fn crash(s: &str, ctxt: &Ctxt) -> ! {
    let l = &ctxt.last_stmt;
    println!("exec IR crashing due to '{s}' at stmt:\n{l}");
    std::process::exit(1);
}
