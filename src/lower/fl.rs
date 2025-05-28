use crate::lower::*;

// This adds higher-level abstractions onto the Ctxt,
// that are relevant for lowering and not only as a general builder.

// context for lowering statements from the AST.
pub(in crate::lower) struct FnLowerCtxt {
    pub loop_stack: Vec<(/*break*/ BlockId, /*continue*/ BlockId)>,

    // the node for the local context.
    // sometimes switched out by the class context.
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
        let d = self.push_table();
        self.build_value_w_dict(payload, type_, d)
    }

    pub fn build_value_w_dict(&mut self, payload: Node, type_: Node, dict: Node) -> Node {
        let t = self.push_table();
        self.push_store_str(t, "payload", payload);
        self.push_store_str(t, "type", type_);
        self.push_store_str(t, "dict", dict);
        t
    }

    pub fn push_return_none(&mut self) {
        let none = self.push_undef();
        let none_ty = self.get_singleton("NoneType");
        let none = self.build_value(none, none_ty);
        self.push_store_str(self.fl().arg_node, "ret", none);
        self.push_return();
    }

    pub fn branch_eq(&mut self, l: Node, r: Node, yes: BlockId, no: BlockId) {
        let cmp = self.push_eq(l, r);
        self.push_if(cmp, yes, no);
    }

    pub fn branch_undef(&mut self, v: Node, undef: BlockId, not_undef: BlockId) {
        let u = self.push_undef();
        self.branch_eq(v, u, undef, not_undef);
    }

    pub fn branch_is_fn(&mut self, v: Node, undef: BlockId, not_undef: BlockId) {
        let u = self.push_undef();
        self.branch_eq(v, u, undef, not_undef);
    }

    pub fn get_singleton(&mut self, v: &str) -> Node {
        self.push_index_str(self.fl().singletons_node, v)
    }
}
