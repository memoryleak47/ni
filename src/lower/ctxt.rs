use crate::*;

pub(in crate::lower) struct Ctxt {
    pub stack: Vec<FnCtxt>,
    pub nameres_tab: NameResTable,
    pub procs: Map<Symbol, Vec<String>>,
}

pub(in crate::lower) struct FnCtxt {
    pub current_pid: Symbol,
    pub lowering: Option<FnLowerCtxt>, // set to None for builtin functions.
}

pub(in crate::lower) struct FnLowerCtxt {
    // pub loop_stack: Vec<(/*break*/ Symbol, /*continue*/ Symbol)>,

    // the original def stmt we are lowering.
    // set to 0 for the main function.
    pub ast_ptr: *const ASTStatement,
}

impl Ctxt {
    pub fn push(&mut self, s: String) {
        let (pid, s_ref) = self.procs.last_mut().unwrap();
        s_ref.push(s);
    }

    pub fn f(&self) -> &FnCtxt {
        self.stack.last().unwrap()
    }

    pub fn f_mut(&mut self) -> &mut FnCtxt {
        self.stack.last_mut().unwrap()
    }

    pub fn fl(&self) -> &FnLowerCtxt {
        self.f().lowering.as_ref().unwrap()
    }

    pub fn fl_mut(&mut self) -> &mut FnLowerCtxt {
        self.f_mut().lowering.as_mut().unwrap()
    }
}
