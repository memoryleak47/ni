use std::fs;

pub const CHECKS: bool = true;
pub const MERGER: bool = false;

pub use indexmap::IndexMap as Map;
pub use indexmap::IndexSet as Set;
pub use std::hash::Hash;

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

pub mod merger_analysis;
pub mod standard_analysis;

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

    match cli.action {
        Action::ShowPostIR => {
            println!("{}", ir_string);
        },
        Action::Run => {
            exec(&ir);
        },
        Action::Analyze => {
            let analyze = if MERGER { merger_analysis::analyze } else { standard_analysis::analyze };
            println!("{}", match analyze(ir) {
                true => "safe",
                false => "unsafe",
            });
        },
        _ => {},
    }
}
