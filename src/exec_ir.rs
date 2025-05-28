use crate::ir::*;

use std::collections::HashMap;

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
    Function(FnId),
    Float(R64),
    Int(i64),
}

#[derive(Debug)]
struct FnCtxt {
    arg: Value,
    nodes: HashMap<Node, Value>,
    fn_id: FnId,
    block_id: BlockId,
    statement_idx: usize,
}

#[derive(Debug)]
struct Ctxt<'ir> {
    heap: Vec<TableData>,
    ir: &'ir IR,
    stack: Vec<FnCtxt>,
    last_stmt: Option<Stmt>, // only for debugging purposes
}

impl<'ir> Ctxt<'ir> {
    fn fcx(&self) -> &FnCtxt {
        self.stack.last().unwrap()
    }

    fn fcx_mut(&mut self) -> &mut FnCtxt {
        self.stack.last_mut().unwrap()
    }
}

#[derive(Default, Debug)]
struct TableData {
    entries: Vec<(Value, Value)>,
}

fn exec_expr(expr: &Expr, ctxt: &mut Ctxt) -> Value {
    match expr {
        Expr::Index(t, idx) => {
            let t = ctxt.fcx().nodes[t].clone();
            let idx = ctxt.fcx().nodes[idx].clone();

            let Value::TablePtr(t) = t else {
                crash(&format!("indexing into non-table {:?}, with index {:?}!", t, idx), ctxt)
            };
            table_get(t, idx, ctxt)
        }
        Expr::Arg => ctxt.fcx().arg.clone(),
        Expr::NewTable => Value::TablePtr(alloc_table(ctxt)),
        Expr::Function(fnid) => Value::Function(*fnid),
        Expr::BinOp(kind, l, r) => {
            let l = ctxt.fcx().nodes[l].clone();
            let r = ctxt.fcx().nodes[r].clone();

            exec_binop(kind.clone(), l, r, ctxt)
        }
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

fn call_fn(f: FnId, arg: Value, ctxt: &mut Ctxt) {
    let fcx = FnCtxt {
        nodes: Default::default(),
        arg,
        fn_id: f,
        block_id: ctxt.ir.fns[&f].start_block,
        statement_idx: 0,
    };
    ctxt.stack.push(fcx);
}

fn alloc_table(ctxt: &mut Ctxt) -> TablePtr {
    let tid = ctxt.heap.len();
    ctxt.heap.push(Default::default());

    tid
}

pub fn exec(ir: &IR) {
    let mut ctxt = Ctxt {
        ir,
        heap: Vec::new(),
        stack: Vec::new(),
        last_stmt: None,
    };

    call_fn(ir.main_fn, Value::Undef, &mut ctxt);

    while ctxt.stack.len() > 0 {
        let l: &FnCtxt = ctxt.stack.last().unwrap();
        ctxt.last_stmt = Some((l.fn_id, l.block_id, l.statement_idx));

        if step(&mut ctxt).is_none() {
            break;
        }
    }
}

fn step_stmt(stmt: &Statement, ctxt: &mut Ctxt) -> Option<()> {
    ctxt.fcx_mut().statement_idx += 1;

    use Statement::*;
    match stmt {
        Compute(n, expr) => {
            let val = exec_expr(expr, ctxt);
            ctxt.fcx_mut().nodes.insert(*n, val);
        }
        Store(t, idx, n) => {
            let t = ctxt.fcx().nodes[t].clone();
            let idx = ctxt.fcx().nodes[idx].clone();
            let val = ctxt.fcx().nodes[n].clone();
            let Value::TablePtr(t) = t.clone() else {
                crash("indexing into non-table!", ctxt)
            };
            table_set(t, idx, val, ctxt);
        }
        If(cond, then_body, else_body) => {
            let cond = ctxt.fcx().nodes[cond].clone();
            let blk = match &cond {
                Value::Bool(true) => then_body,
                Value::Bool(false) => else_body,
                _ => crash("UB: non-boolean in if-condition", ctxt),
            };
            ctxt.fcx_mut().block_id = *blk;
            ctxt.fcx_mut().statement_idx = 0;
        }
        FnCall(f, arg) => {
            let f = ctxt.fcx().nodes[f].clone();
            let arg = ctxt.fcx().nodes[arg].clone();

            match f {
                Value::Function(f_id) => call_fn(f_id, arg, ctxt),
                v => crash(&format!("trying to execute non-function value! {:?}", v), ctxt),
            };
        }
        Print(n) => {
            let val = &ctxt.fcx().nodes[n];
            match val {
                Value::Undef => crash("print called on Undef!", ctxt),
                Value::Bool(true) => println!("True"),
                Value::Bool(false) => println!("False"),
                Value::Str(s) => println!("{}", s),
                Value::TablePtr(ptr) => println!("table: {}", ptr),
                Value::Function(fid) => println!("function: {}", fid),
                Value::Float(x) => println!("{}", x),
                Value::Int(x) => println!("{}", x),
            }
        }
        Throw(s) => {
            println!("ERROR: {}", s);
            return None;
        }
        Return => {
            ctxt.stack.pop();
        }
    }

    Some(())
}

fn step(ctxt: &mut Ctxt) -> Option<()> {
    let l: &FnCtxt = ctxt.stack.last().unwrap();
    let stmt = ctxt.ir.fns[&l.fn_id].blocks[&l.block_id]
        .get(l.statement_idx)
        .unwrap_or_else(|| crash("stmt overflow", ctxt));
    step_stmt(stmt, ctxt)
}

fn stringify_stmt(pos: Stmt, ctxt: &Ctxt) -> String {
    ctxt.ir.fns[&pos.0].blocks[&pos.1]
        .get(pos.2)
        .map(|x| format!("{x}"))
        .unwrap_or_else(|| "<empty>".to_string())
}

fn stringify_last_stmt(ctxt: &Ctxt) -> String {
    ctxt.last_stmt.map(|x| stringify_stmt(x, ctxt))
        .unwrap_or_else(|| "<no last stmt>".to_string())
}

fn crash(s: &str, ctxt: &Ctxt) -> ! {
    let stmt = stringify_last_stmt(ctxt);
    println!("exec IR crashing due to '{s}' at stmt {stmt}");
    std::process::exit(1);
}
