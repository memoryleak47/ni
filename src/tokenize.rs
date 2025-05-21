use crate::*;

#[derive(Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Int(i64),
    Str(String),
    Bool(bool),
    None,
    Colon,
    LParen,
    RParen,
    Comma,
    Equals,
    If,
    While,
    Return,
    Break,
    Continue,
    Def,
    Class,
    Pass,
    Scope(ScopeKind),
    Newline,
    Indent,
    Unindent,
    BinOp(BinOpKind),
}

fn ident_char(c: char) -> bool {
    ('0'..='9').contains(&c) || ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_'
}

fn int_char(c: char) -> bool {
    ('0'..='9').contains(&c)
}

enum TokenizerState {
    CountingIndents(usize),
    InLine,
    InInt(String),
    InStr(char, String),
    InIdent(String),
    InComment, // #
}

pub fn tokenize(s: &str) -> Vec<Token> {
    let mut chars: Vec<_> = s.chars().collect();
    chars.push('#'); // automatically closes off final idents.

    let mut i = 0;
    let mut tokens = Vec::new();

    // how many indents where in the last line.
    let mut last_indent = 0;

    let mut state = TokenizerState::CountingIndents(0);

    while i < chars.len() {
        let c = chars[i];
        match state {
            TokenizerState::CountingIndents(n) => {
                if c == '\t' {
                    state = TokenizerState::CountingIndents(n + 1);
                    i += 1;
                } else if c == '\n' {
                    // this ignores empty lines.
                    state = TokenizerState::CountingIndents(0);
                    i += 1;
                } else {
                    state = TokenizerState::InLine;
                    let delta = (n as i32) - (last_indent as i32);

                    for _ in 0..delta {
                        tokens.push(Token::Indent);
                    }
                    for _ in delta..0 {
                        tokens.push(Token::Unindent);
                    }

                    last_indent = n;
                }
            }
            TokenizerState::InLine => match &chars[i..] {
                ['+', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Plus));
                    i += 1;
                }
                ['-', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Minus));
                    i += 1;
                }
                ['*', '*', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Pow));
                    i += 2;
                }
                ['*', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Mul));
                    i += 1;
                }
                ['/', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Div));
                    i += 1;
                }
                ['%', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Mod));
                    i += 1;
                }
                ['<', '=', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Le));
                    i += 2;
                }
                ['>', '=', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Ge));
                    i += 2;
                }
                ['=', '=', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::IsEqual));
                    i += 2;
                }
                ['!', '=', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::IsNotEqual));
                    i += 2;
                }
                ['<', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Lt));
                    i += 1;
                }
                ['>', ..] => {
                    tokens.push(Token::BinOp(BinOpKind::Gt));
                    i += 1;
                }

                ['"', ..] => {
                    state = TokenizerState::InStr('"', String::new());
                    i += 1;
                }
                ['\'', ..] => {
                    state = TokenizerState::InStr('\'', String::new());
                    i += 1;
                }
                ['\n', ..] => {
                    state = TokenizerState::CountingIndents(0);
                    i += 1;
                }
                ['#', ..] => {
                    state = TokenizerState::InComment;
                    i += 1;
                }
                [':', ..] => {
                    tokens.push(Token::Colon);
                    i += 1;
                }
                ['(', ..] => {
                    tokens.push(Token::LParen);
                    i += 1;
                }
                [')', ..] => {
                    tokens.push(Token::RParen);
                    i += 1;
                }
                [',', ..] => {
                    tokens.push(Token::Comma);
                    i += 1;
                }
                ['=', ..] => {
                    tokens.push(Token::Equals);
                    i += 1;
                }

                [' ', ..] => {
                    i += 1;
                }
                _ if int_char(c) => {
                    state = TokenizerState::InInt(c.to_string());
                    i += 1;
                }
                _ if ident_char(c) => {
                    state = TokenizerState::InIdent(c.to_string());
                    i += 1;
                }
                _ => panic!("unknown char '{c}'"),
            },
            TokenizerState::InStr(delim, mut s) => {
                if c == delim {
                    tokens.push(Token::Str(s));
                    state = TokenizerState::InLine;
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
                        "if" => Token::If,
                        "while" => Token::While,
                        "return" => Token::Return,
                        "break" => Token::Break,
                        "continue" => Token::Continue,
                        "def" => Token::Def,
                        "class" => Token::Class,
                        "pass" => Token::Pass,
                        "True" => Token::Bool(true),
                        "False" => Token::Bool(false),
                        "None" => Token::None,
                        "global" => Token::Scope(ScopeKind::Global),
                        "nonlocal" => Token::Scope(ScopeKind::NonLocal),
                        _ => Token::Ident(s),
                    });
                    state = TokenizerState::InLine;
                }
            }
            TokenizerState::InInt(mut s) => {
                if int_char(c) {
                    s.push(c);
                    state = TokenizerState::InInt(s);
                    i += 1;
                } else {
                    let int = s.parse().expect("Can't parse int!");
                    tokens.push(Token::Int(int));
                    state = TokenizerState::InLine;
                }
            }
            TokenizerState::InComment => {
                if c == '\n' {
                    state = TokenizerState::CountingIndents(0);
                    i += 1;
                } else {
                    i += 1;
                }
            }
        }
    }

    tokens
}
