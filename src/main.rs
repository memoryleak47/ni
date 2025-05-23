use std::fs;

pub use std::collections::HashMap as Map;
pub use std::collections::HashSet as Set;

pub use noisy_float::prelude::{Float, R64};

mod ast;
pub use ast::*;

mod tokenize;
pub use tokenize::*;

mod assemble;
pub use assemble::*;

mod nameres;
pub use nameres::*;

mod lower;
pub use lower::*;

mod ir;
pub use ir::*;

mod exec_ir;
pub use exec_ir::*;

mod fmt;
pub use fmt::*;

mod cli;
pub use cli::*;

fn main() {
    let cli = cli();
    let contents = fs::read_to_string(cli.filename).unwrap();
    let toks = tokenize(&contents);
    if let Action::ShowTokens = cli.action {
        println!("{:?}", toks);
        return;
    }

    let ast = assemble(&toks);

    if let Action::ShowAst = cli.action {
        println!("{:?}", ast);
        return;
    }

    let ir = lower(&ast);

    if let Action::ShowIR = cli.action {
        println!("{}", ir);
        return;
    }

    exec(&ir);
}
