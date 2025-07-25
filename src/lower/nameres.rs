use crate::*;

#[derive(Debug)]
pub enum VarPlace {
    Global, // unknown: check global module namespace and then builtins.
    // TODO add
    // Closured(/*how many scopes out?*/ usize),
    Local,
}

pub type NameResTable = Map<
    (
        /*ptr address of ASTStatement::Def(_)*/ *const ASTStatement,
        String,
    ),
    VarPlace,
>;

pub fn nameres(ast: &AST) -> NameResTable {
    let mut nrt = NameResTable::new();
    iter(ast, &mut nrt, 0 as _);
    nrt
}

fn iter(ast: &AST, nrt: &mut NameResTable, current_fn_ptr: *const ASTStatement) {
    for stmt in ast {
        match stmt {
            ASTStatement::Assign(ASTExpr::Var(v), _) => {
                let k = (current_fn_ptr, v.to_string());
                if !nrt.contains_key(&k) {
                    nrt.insert(k, VarPlace::Local);
                }
            }
            ASTStatement::Assign(..) => {}, // correct?
            ASTStatement::Def(name, args, body) => {
                let k = (current_fn_ptr, name.to_string());
                if !nrt.contains_key(&k) {
                    nrt.insert(k, VarPlace::Local);
                }

                for a in args {
                    let k = (stmt as _, a.to_string());
                    if !nrt.contains_key(&k) {
                        nrt.insert(k, VarPlace::Local);
                    }
                }
                iter(body, nrt, stmt as _);
            }
            ASTStatement::Class(name, _args, body) => {
                let k = (current_fn_ptr, name.to_string());
                if !nrt.contains_key(&k) {
                    nrt.insert(k, VarPlace::Local);
                }

                iter(body, nrt, stmt as _);
            },
            ASTStatement::If(_, body) | ASTStatement::While(_, body) => {
                iter(body, nrt, current_fn_ptr);
            }
            ASTStatement::For(v, _, body) => {
                let k = (current_fn_ptr, v.to_string());
                if !nrt.contains_key(&k) {
                    nrt.insert(k, VarPlace::Local);
                }
                iter(body, nrt, current_fn_ptr);
            }
            ASTStatement::Try(body, excepts) => {
                iter(body, nrt, current_fn_ptr);
                for except in excepts {
                    iter(&except.body, nrt, current_fn_ptr);
                }
            }
            ASTStatement::Scope(ScopeKind::Global, vars) => {
                for v in vars {
                    nrt.insert((current_fn_ptr, v.to_string()), VarPlace::Global);
                }
            }
            ASTStatement::Scope(ScopeKind::NonLocal, vars) => {
                for v in vars {
                    nrt.insert((current_fn_ptr, v.to_string()), todo!());
                }
            }
            ASTStatement::Break
            | ASTStatement::Continue
            | ASTStatement::Return(_)
            | ASTStatement::Expr(_)
            | ASTStatement::Raise(_)
            | ASTStatement::Pass => {}

            _ => todo!(),
        }
    }
}
