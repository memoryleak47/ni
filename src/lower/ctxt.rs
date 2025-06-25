use crate::*;

pub(in crate::lower) struct Ctxt {
    pub stack: Vec<FnCtxt>,
    pub nameres_tab: NameResTable,
    pub ir: IR,
}

pub(in crate::lower) struct FnCtxt {
    pub node_ctr: usize,
    pub current_fn: ProcId,
    pub current_blk: ProcId,
    pub lowering: Option<FnLowerCtxt>, // set to None for builtin functions.
}

pub(in crate::lower) struct FnLowerCtxt {
    pub loop_stack: Vec<(/*break*/ ProcId, /*continue*/ ProcId)>,

    // the original def stmt we are lowering.
    // set to 0 for the main function.
    pub ast_ptr: *const ASTStatement,
}
