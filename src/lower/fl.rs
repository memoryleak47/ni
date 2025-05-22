use crate::lower::*;

// This adds higher-level abstractions onto the Ctxt,
// that are relevant for lowering and not only as a general builder.

// context for lowering statements from the AST.
pub(in crate::lower) struct FnLowerCtxt {
    pub loop_stack: Vec<(/*break*/ BlockId, /*continue*/ BlockId)>,
    pub namespace_node: Node,
    pub global_node: Node,
    pub singletons_node: Node,
    pub arg_node: Node,

    // the original def stmt we are lowering.
    // set to 0 for the main function.
    pub ast_ptr: *const ASTStatement,
}

impl Ctxt {
    pub fn fl(&self) -> &FnLowerCtxt {
        self.f().lowering.as_ref().unwrap()
    }

    pub fn fl_mut(&mut self) -> &mut FnLowerCtxt {
        self.f_mut().lowering.as_mut().unwrap()
    }

    pub fn build_value(&mut self, payload: Node, type_: Node) -> Node {
        let t = self.push_table();
        self.push_store_str(t, "type", type_);
        let dict = self.push_table();
        self.push_store_str(t, "payload", payload);
        self.push_store_str(t, "dict", dict);
        t
    }

    pub fn push_return_none(&mut self) {
        let none = self.push_none();
        self.push_store_str(self.fl().arg_node, "ret", none);
        self.push_return();
    }

    pub fn branch_undef(&mut self, v: Node, undef: BlockId, not_undef: BlockId) {
        let u = self.push_undef();
        let cmp = self.push_eq(v, u);
        self.push_if(cmp, undef, not_undef);
    }

    pub fn get_singleton(&mut self, v: &str) -> Node {
        self.push_index_str(self.fl().singletons_node, v)
    }
}
