use std::env;
use std::fs;
use std::fmt;

// --- ERROR TYPE ---

#[derive(Debug)]
struct LangError {
	message: String,
	line: usize,
	col: usize,
}

impl LangError {
	fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
		LangError { message: message.into(), line, col }
	}
}

impl fmt::Display for LangError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "[{}:{}] Error: {}", self.line, self.col, self.message)
	}
}

// --- LEXER ---

#[derive(Debug, PartialEq, Clone)]
enum TokenKind {
	Print,          //prints stuff (duh)
	Let,            // variable definition
	If,             // if keyword
	Else,           // else keyword

	// data types
	String,
	Number,

	Id, // variable name
	Assign, // assigns value to a variable ( = )

	// math
	Plus,   // +
	Minus,  // -
	Star,   // *
	Slash,  // /

	// logical operators
	Eq,     // ==
	NotEq,  // !=
	Lt,     // <
	Gt,     // >
	LtEq,   // <=
	GtEq,   // >=
	And,    // &&
	Or,     // ||
	Not,    // !
	LParen, // (
	RParen, // )
	LBrace, // {
	RBrace, // }
}

#[derive(Debug, Clone)]
struct Token {  //Pairs the data type and the value from source code
	kind: TokenKind,
	value: String,
	line: usize,
	col: usize,
}

fn lexer(code: &str) -> Result<Vec<Token>, LangError> {
	let mut tokens = Vec::new();
	let mut chars = code.chars().peekable();
	let mut line = 1usize;
	let mut col = 1usize;

	while let Some(&ch) = chars.peek() {
		match ch {
			'\n' => { chars.next(); line += 1; col = 1; }                                             //If character is Empty, Skip
			' ' | '\t' | '\r' => { chars.next(); col += 1; }

			'"' => {                                                                                		//If char is ", loops through following chars until hitting the next "
				let start_col = col;
				chars.next(); col += 1;
				let mut s = String::from('"');
				let mut closed = false;
				while let Some(&c) = chars.peek() {
					chars.next(); col += 1;
					if c == '"' { s.push('"'); closed = true; break; }
					if c == '\n' { line += 1; col = 1; }
					s.push(c);
				}
				if !closed {
					return Err(LangError::new("Unterminated string literal", line, start_col));
				}
				tokens.push(Token { kind: TokenKind::String, value: s, line, col: start_col });
			}

			c if c.is_ascii_digit() => {                                                      		//If char is a Number
				let start_col = col;
				let mut num = String::new();
				let mut dots = 0u8;
				while let Some(&c) = chars.peek() {
					if c.is_ascii_digit() { num.push(c); chars.next(); col += 1; }
					else if c == '.' {
						dots += 1;
						if dots > 1 {
							return Err(LangError::new("Invalid number literal: multiple decimal points", line, col));
						}
						num.push(c); chars.next(); col += 1;
					}
					else { break; }
				}
				tokens.push(Token { kind: TokenKind::Number, value: num, line, col: start_col });
			}

			c if c.is_alphabetic() || c == '_' => {                                           		//If char is Alphabetic, check if the word is a keyword afterward, else it's a var name
				let start_col = col;
				let mut word = String::new();
				while let Some(&c) = chars.peek() {
					if c.is_alphanumeric() || c == '_' { word.push(c); chars.next(); col += 1; }
					else { break; }
				}
				let kind = match word.as_str() {
					"print" => TokenKind::Print,
					"let"   => TokenKind::Let,
					"if"    => TokenKind::If,
					"else"  => TokenKind::Else,
					_       => TokenKind::Id,
				};
				tokens.push(Token { kind, value: word, line, col: start_col });
			}

			'=' => {                                                                                		//If next char is also =, it's a comparing ==, else it's an assigning =
				let start_col = col;
				chars.next(); col += 1;
				if chars.peek() == Some(&'=') {
					chars.next(); col += 1;
					tokens.push(Token { kind: TokenKind::Eq, value: String::from("=="), line, col: start_col });
				} else {
					tokens.push(Token { kind: TokenKind::Assign, value: String::from("="), line, col: start_col });
				}
			}

			'!' => {                                                                                		//Logical Negation
				let start_col = col;
				chars.next(); col += 1;
				if chars.peek() == Some(&'=') {
					chars.next(); col += 1;
					tokens.push(Token { kind: TokenKind::NotEq, value: String::from("!="), line, col: start_col });
				} else {
					tokens.push(Token { kind: TokenKind::Not, value: String::from("!"), line, col: start_col });
				}
			}

			'<' => {
				let start_col = col;
				chars.next(); col += 1;
				if chars.peek() == Some(&'=') {
					chars.next(); col += 1;
					tokens.push(Token { kind: TokenKind::LtEq, value: String::from("<="), line, col: start_col });
				} else {
					tokens.push(Token { kind: TokenKind::Lt, value: String::from("<"), line, col: start_col });
				}
			}

			'>' => {
				let start_col = col;
				chars.next(); col += 1;
				if chars.peek() == Some(&'=') {
					chars.next(); col += 1;
					tokens.push(Token { kind: TokenKind::GtEq, value: String::from(">="), line, col: start_col });
				} else {
					tokens.push(Token { kind: TokenKind::Gt, value: String::from(">"), line, col: start_col });
				}
			}

			'&' => {
				let start_col = col;
				chars.next(); col += 1;
				if chars.peek() == Some(&'&') {
					chars.next(); col += 1;
					tokens.push(Token { kind: TokenKind::And, value: String::from("&&"), line, col: start_col });
				} else {
					return Err(LangError::new("Expected '&&', got single '&'", line, start_col));
				}
			}

			'|' => {
				let start_col = col;
				chars.next(); col += 1;
				if chars.peek() == Some(&'|') {
					chars.next(); col += 1;
					tokens.push(Token { kind: TokenKind::Or, value: String::from("||"), line, col: start_col });
				} else {
					return Err(LangError::new("Expected '||', got single '|'", line, start_col));
				}
			}

			'+' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Plus,   value: String::from("+"), line, col: c }); }
			'-' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Minus,  value: String::from("-"), line, col: c }); }
			'*' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Star,   value: String::from("*"), line, col: c }); }
			'/' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Slash,  value: String::from("/"), line, col: c }); }
			'(' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::LParen, value: String::from("("), line, col: c }); }
			')' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::RParen, value: String::from(")"), line, col: c }); }
			'{' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::LBrace, value: String::from("{"), line, col: c }); }
			'}' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::RBrace, value: String::from("}"), line, col: c }); }

			other => return Err(LangError::new(format!("Unexpected character '{}'", other), line, col)),
		}
	}

	Ok(tokens)
}

// --- TRANSPILER ---

struct Transpiler {                                                                                 		//Owns the token list and tracks the current position
	tokens: Vec<Token>,
	pos: usize,
}

impl Transpiler {
	fn new(tokens: Vec<Token>) -> Self {
		Transpiler { tokens, pos: 0 }
	}

	fn peek(&self) -> Option<&Token> {                                                              		//Returns a reference without moving the position
		self.tokens.get(self.pos)
	}

	fn consume(&mut self) -> Option<Token> {																//Returns a Clone of the current token and moves the counter
		let token = self.tokens.get(self.pos).cloned();
		self.pos += 1;
		token
	}

	fn expect(&mut self, kind: TokenKind, context: &str) -> Result<Token, LangError> {					// Consumes a token, errors if it isn't the expected kind
		match self.consume() {
			Some(t) if t.kind == kind => Ok(t),
			Some(t) => Err(LangError::new(
				format!("Expected {:?} {}, got {:?} ('{}')", kind, context, t.kind, t.value),
				t.line, t.col,
			)),
			None => Err(LangError::new(
				format!("Expected {:?} {}, got end of file", kind, context),
				0, 0,
			)),
		}
	}

	fn parse_primary(&mut self) -> Result<String, LangError> {												//Parses a single primary: literal, variable, unary op, or grouped expression
		match self.peek().map(|t| t.kind.clone()) {
			Some(TokenKind::LParen) => {
				self.consume();
				let expr = self.parse_expr(0)?;
				self.expect(TokenKind::RParen, "to close '('")?;
				Ok(format!("({})", expr))
			}
			Some(TokenKind::Not) => {
				self.consume();
				let operand = self.parse_primary()?;
				Ok(format!("!{}", operand))
			}
			Some(TokenKind::Minus) => {
				self.consume();
				let operand = self.parse_primary()?;
				Ok(format!("-{}", operand))
			}
			Some(TokenKind::Id) => {																		//Adds the $ before a variable
				let t = self.consume().unwrap();
				Ok(format!("${}", t.value))
			}
			Some(TokenKind::String) | Some(TokenKind::Number) => {
				Ok(self.consume().unwrap().value)
			}
			Some(_) => {
				let t = self.peek().unwrap();
				Err(LangError::new(
					format!("Unexpected token '{}'  in expression", t.value),
					t.line, t.col,
				))
			}
			None => Err(LangError::new("Unexpected end of file in expression", 0, 0)),
		}
	}

	fn op_precedence(kind: &TokenKind) -> Option<u8> {														//Basic order of operations
		match kind {
			TokenKind::Or                                           				=> Some(1),
			TokenKind::And                                          				=> Some(2),
			TokenKind::Eq | TokenKind::NotEq                       					=> Some(3),
			TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq		        => Some(4),
			TokenKind::Plus | TokenKind::Minus                     					=> Some(5),
			TokenKind::Star | TokenKind::Slash                     					=> Some(6),
			_                                                       				=> None,
		}
	}

	fn parse_expr(&mut self, min_prec: u8) -> Result<String, LangError> {
		let mut left = self.parse_primary()?;

		loop {
			let prec = match self.peek() {
				Some(t) => match Self::op_precedence(&t.kind) {
					Some(p) if p >= min_prec => p,
					_ => break,
				},
				None => break,
			};

			let op = self.consume().unwrap().value;
			let right = self.parse_expr(prec + 1)?;
			left = format!("{} {} {}", left, op, right);
		}

		Ok(left)
	}

	fn parse_block(&mut self) -> Result<String, LangError> {												// Parses { ... } and returns the inner statements indented
		self.expect(TokenKind::LBrace, "to open block")?;
		let mut body = String::new();
		while let Some(t) = self.peek() {
			if t.kind == TokenKind::RBrace { break; }
			let stmt = self.statement()?;
			if !stmt.is_empty() {
				body.push_str(&format!("\t{};\n", stmt));
			}
		}
		self.expect(TokenKind::RBrace, "to close block")?;
		Ok(body)
	}

	fn statement(&mut self) -> Result<String, LangError> {													//for now, only two statements, Let and Print
		match self.peek().map(|t| t.kind.clone()) {
			Some(TokenKind::Let) => {																		//expects an ID token, an Assign token and an Expression
				self.consume();
				let name_tok = match self.consume() {
					Some(t) if t.kind == TokenKind::Id => t,
					Some(t) => return Err(LangError::new(
						format!("Expected variable name after 'let', got '{}'", t.value),
						t.line, t.col,
					)),
					None => return Err(LangError::new("Expected variable name after 'let', got end of file", 0, 0)),
				};
				self.expect(TokenKind::Assign, "after variable name")?;
				let expr = self.parse_expr(0)?;
				Ok(format!("${} = {}", name_tok.value, expr))
			}
			Some(TokenKind::Print) => {																		//prints the expression after it
				self.consume();
				let expr = self.parse_expr(0)?;
				Ok(format!("echo {}", expr))
			}
			Some(TokenKind::If) => {																		// if (cond) { ... } else { ... }
				self.consume();
				self.expect(TokenKind::LParen, "after 'if'")?;
				let cond = self.parse_expr(0)?;
				self.expect(TokenKind::RParen, "to close 'if' condition")?;
				let if_body = self.parse_block()?;
				let else_part = if self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Else) {
					self.consume();
					let else_body = self.parse_block()?;
					format!(" else {{\n{}}}", else_body)
				} else {
					String::new()
				};
				Ok(format!("if ({}) {{\n{}}}{}", cond, if_body, else_part))
			}
			Some(_) => {
				let t = self.peek().unwrap();
				Err(LangError::new(
					format!("Unexpected token '{}' at start of statement", t.value),
					t.line, t.col,
				))
			}
			None => Ok(String::new()),
		}
	}

	fn transpile(&mut self) -> Result<String, LangError> {
		let mut output = String::from("<?php\n\n");												//starts PHP
		while self.pos < self.tokens.len() {
			let stmt = self.statement()?;														//calls statement() until there's no more tokens left
			if !stmt.is_empty() {
				output.push_str(&stmt);
				if stmt.starts_with("if") {
					output.push('\n');
				} else {
					output.push_str(";\n");
				}
			}
		}
		Ok(output)
	}
}

// --- MAIN ---

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		eprintln!("Usage: ezlang <filename.ez>");
		return;
	}

	let filename = &args[1];

	let code = match fs::read_to_string(filename) {
		Ok(c) => c,
		Err(_) => { eprintln!("Error: File '{}' not found.", filename); return; }
	};

	let tokens = match lexer(&code) {
		Ok(t) => t,
		Err(e) => { eprintln!("{}", e); return; }
	};

	let mut transpiler = Transpiler::new(tokens);
	let php_result = match transpiler.transpile() {
		Ok(r) => r,
		Err(e) => { eprintln!("{}", e); return; }
	};

	let output_filename = filename.replace(".ez", ".php");
	match fs::write(&output_filename, &php_result) {
		Ok(_) => println!("Successfully compiled {} to {}", filename, output_filename),
		Err(e) => eprintln!("Failed to write output: {}", e),
	}
}
