pub enum Action {
    ShowTokens,
    ShowAst,
    ShowIR,
    ShowPostIR,
    Run,
}

pub struct CliConfig {
    pub action: Action,
    pub filename: String,
}

pub fn cli() -> CliConfig {
    let mut cli = CliConfig {
        action: Action::Run,
        filename: String::new(),
    };
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    if get_flag("--show-tokens", &mut args) {
        cli.action = Action::ShowTokens;
    }
    if get_flag("--show-ast", &mut args) {
        cli.action = Action::ShowAst;
    }
    if get_flag("--show-ir", &mut args) {
        cli.action = Action::ShowIR;
    }
    if get_flag("--show-post-ir", &mut args) {
        cli.action = Action::ShowPostIR;
    }

    let [ref filename] = *args else {
        panic!("wrong number of CLI args")
    };
    cli.filename = filename.clone();
    cli
}

fn get_flag(flag: &str, args: &mut Vec<String>) -> bool {
    if args.iter().any(|x| x == flag) {
        args.retain(|x| x != flag);
        return true;
    }
    false
}
