use std::fs;

pub use indexmap::IndexMap as Map;
pub use indexmap::IndexSet as Set;

pub use noisy_float::prelude::{Float, R64};

mod ast;
pub use ast::*;

mod symbol;
pub use symbol::*;

mod lower;
pub use lower::*;

mod ir;
pub use ir::*;

mod cli;
pub use cli::*;

fn main() {
    let cli = cli();
    let contents = fs::read_to_string(&cli.filename).unwrap();
    let ir_string = if !cli.filename.ends_with(".ir") {
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

        let ir_string = lower(&ast);
        if let Action::ShowIR = cli.action {
            println!("{}", ir_string);
            return;
        }

        ir_string
    } else {
        contents
    };
    let toks = ir_tokenize(&ir_string);
    let ir = ir_assemble(&toks[..]);

    if let Action::ShowPostIR = cli.action {
        println!("{}", ir_string);
        return;
    }

    exec(&ir);
}
