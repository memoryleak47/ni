pub enum Token {
	Ident(String), Num(f64),
	Colon, LParen, RParen, Comma,
	If, While, Return, Break, Continue, Def
	Newline, Indent, Unindent,
}

pub fn tokenize(s: &str) -> Vec<Token> {
	
}
