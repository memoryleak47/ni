#[derive(Debug)]
pub enum Token {
	Ident(String), Num(f64),
	Colon, LParen, RParen, Comma,
	If, While, Return, Break, Continue, Def,
	Newline, Indent, Unindent,
}

fn ident_char(c: char) -> bool {
	('0'..='9').contains(&c) || ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_'
}

enum TokenizerState {
	CountingIndents(usize),
	InLine,
	InStr(String),
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
					state = TokenizerState::CountingIndents(n+1);
					i += 1;
				} else {
					state = TokenizerState::InLine;
					let delta = (n as i32) - (last_indent as i32);

					for _ in 0..delta { tokens.push(Token::Indent); }
					for _ in delta..0 { tokens.push(Token::Unindent); }

					last_indent = n;
				}
			},
			TokenizerState::InLine => {
				match c {
					'\n' => { state = TokenizerState::CountingIndents(0); i += 1; },
					'#' => { state = TokenizerState::InComment; i += 1; },
					':' => { tokens.push(Token::Colon); i += 1; },
					'(' => { tokens.push(Token::LParen); i += 1; },
					')' => { tokens.push(Token::RParen); i += 1; },
					',' => { tokens.push(Token::Comma); i += 1; },
					' ' => { i += 1; },
					_ if ident_char(c) => { state = TokenizerState::InStr(c.to_string()); i += 1; },
					_ => panic!("unknown char '{c}'"),
				}
			},
			TokenizerState::InStr(mut s) => {
				if ident_char(c) {
					s.push(c);
					state = TokenizerState::InStr(s);
					i += 1;
				} else {
					tokens.push(match &*s {
						"if" => Token::If,
						"while" => Token::While,
						"return" => Token::Return,
						"break" => Token::Break,
						"continue" => Token::Continue,
						"def" => Token::Def,
						_ => Token::Ident(s),
					});
					state = TokenizerState::InLine;
				}
			},
			TokenizerState::InComment => {
				if c == '\n' {
					state = TokenizerState::CountingIndents(0);
					i += 1;
				} else { i += 1; }
			},
		}

	}

	tokens
}
