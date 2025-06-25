use crate::*;

mod nameres;
pub use nameres::*;

fn lower_ast(ast: &AST) -> String {
    let nameres_tab = nameres(ast);

    String::new()
}

pub fn lower(ast: &AST) -> IR {
    let mut s = lower_ast(ast);
    s.extend(include_str!("../sem/init.ir").chars());

    let toks = ir_tokenize(&s);
    ir_assemble(&toks[..])
}
