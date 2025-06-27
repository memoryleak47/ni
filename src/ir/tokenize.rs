use crate::*;

#[derive(Debug, PartialEq)]
pub enum IRToken {
    Symbol(Symbol),
    Int(i64),
    Str(String),

    LParen, RParen,
    LBracket, RBracket,
    LBrace, RBrace,
    Comma, Dot, Equals, Semicolon, At,

    Proc, Exit, Panic, Jmp, Main, Print,
    BinOp(BinOpKind),
}

fn ident_char(c: char) -> bool {
    ('0'..='9').contains(&c) || ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_'
}

fn int_char(c: char) -> bool {
    ('0'..='9').contains(&c)
}

enum TokenizerState {
    None,
    InInt(String),
    InStr(char, String),
    InIdent(String),
    InComment, // #
}

pub fn ir_tokenize(s: &str) -> Vec<IRToken> {
    let mut chars: Vec<_> = s.chars().collect();
    chars.push('#'); // automatically closes off final idents.

    let mut i = 0;
    let mut tokens = Vec::new();

    let mut state = TokenizerState::None;

    while i < chars.len() {
        let c = chars[i];
        match state {
            TokenizerState::None => match &chars[i..] {
                ['+', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Plus)); i += 1; }
                ['-', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Minus)); i += 1; }
                ['*', '*', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Pow)); i += 2; }
                ['*', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Mul)); i += 1; }
                ['/', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Div)); i += 1; }
                ['%', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Mod)); i += 1; }
                ['<', '=', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Le)); i += 2; }
                ['>', '=', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Ge)); i += 2; }
                ['<', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Lt)); i += 1; }
                ['>', ..] => { tokens.push(IRToken::BinOp(BinOpKind::Gt)); i += 1; }
                ['"', ..] => { state = TokenizerState::InStr('"', String::new()); i += 1; }
                ['\'', ..] => { state = TokenizerState::InStr('\'', String::new()); i += 1; }
                ['#', ..] => { state = TokenizerState::InComment; i += 1; }
                ['(', ..] => { tokens.push(IRToken::LParen); i += 1; }
                [')', ..] => { tokens.push(IRToken::RParen); i += 1; }
                ['[', ..] => { tokens.push(IRToken::LBracket); i += 1; }
                [']', ..] => { tokens.push(IRToken::RBracket); i += 1; }
                ['{', ..] => { tokens.push(IRToken::LBrace); i += 1; }
                ['}', ..] => { tokens.push(IRToken::RBrace); i += 1; }
                [',', ..] => { tokens.push(IRToken::Comma); i += 1; }
                ['=', ..] => { tokens.push(IRToken::Equals); i += 1; }
                [';', ..] => { tokens.push(IRToken::Semicolon); i += 1; }
                ['@', ..] => { tokens.push(IRToken::At); i += 1; }
                ['.', ..] => { tokens.push(IRToken::Dot); i += 1; }

                _ if int_char(c) => { state = TokenizerState::InInt(c.to_string()); i += 1; },
                _ if ident_char(c) => { state = TokenizerState::InIdent(c.to_string()); i += 1; },
                _ if c.is_whitespace() => { i += 1; },
                _ => panic!("unknown char '{c}'"),
            },
            TokenizerState::InStr(delim, mut s) => {
                if c == delim {
                    tokens.push(IRToken::Str(s));
                    state = TokenizerState::None;
                } else {
                    s.push(c);
                    state = TokenizerState::InStr(delim, s);
                }
                i += 1;
            }
            TokenizerState::InIdent(mut s) => {
                if ident_char(c) {
                    s.push(c);
                    state = TokenizerState::InIdent(s);
                    i += 1;
                } else {
                    tokens.push(match &*s {
                        "proc" => IRToken::Proc,
                        "exit" => IRToken::Exit,
                        "panic" => IRToken::Panic,
                        "jmp" => IRToken::Jmp,
                        "main" => IRToken::Main,
                        "print" => IRToken::Print,
                        _ => IRToken::Symbol(Symbol::new(s)),
                    });
                    state = TokenizerState::None;
                }
            }
            TokenizerState::InInt(mut s) => {
                if int_char(c) {
                    s.push(c);
                    state = TokenizerState::InInt(s);
                    i += 1;
                } else {
                    let int = s.parse().expect("Can't parse int!");
                    tokens.push(IRToken::Int(int));
                    state = TokenizerState::None;
                }
            }
            TokenizerState::InComment => {
                if c == '\n' {
                    state = TokenizerState::None;
                }
                i += 1;
            }
        }
    }

    tokens
}
