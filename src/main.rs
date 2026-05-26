use std::env;
use std::fs;

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
}

fn lexer(code: &str) -> Result<Vec<Token>, String> {
	let mut tokens = Vec::new();
	let mut chars = code.chars().peekable();

	while let Some(&ch) = chars.peek() {
		match ch {
			' ' | '\t' | '\n' | '\r' => { chars.next(); }                                     		//If character is Empty, Skip

			'"' => {                                                                                		//If char is ", loops through following chars until hitting the next "
				chars.next();
				let mut s = String::from('"');
				while let Some(&c) = chars.peek() {
					chars.next();
					if c == '"' { s.push('"'); break; }
					s.push(c);
				}
				tokens.push(Token { kind: TokenKind::String, value: s });
			}

			c if c.is_ascii_digit() => {                                                      		//If char is a Number
				let mut num = String::new();
				while let Some(&c) = chars.peek() {
					if c.is_ascii_digit() || c == '.' { num.push(c); chars.next(); }
					else { break; }
				}
				tokens.push(Token { kind: TokenKind::Number, value: num });
			}

			c if c.is_alphabetic() || c == '_' => {                                           		//If char is Alphabetic, check if the word is a keyword afterward, else it's a var name
				let mut word = String::new();
				while let Some(&c) = chars.peek() {
					if c.is_alphanumeric() || c == '_' { word.push(c); chars.next(); }
					else { break; }
				}
				let kind = match word.as_str() {
					"print" => TokenKind::Print,
					"let"   => TokenKind::Let,
					"if"    => TokenKind::If,
					"else"  => TokenKind::Else,
					_       => TokenKind::Id,
				};
				tokens.push(Token { kind, value: word });
			}

			'=' => {                                                                                		//If next char is also =, it's a comparing ==, else it's an assigning =
				chars.next();
				if chars.peek() == Some(&'=') {
					chars.next();
					tokens.push(Token { kind: TokenKind::Eq, value: String::from("==") });
				} else {
					tokens.push(Token { kind: TokenKind::Assign, value: String::from("=") });
				}
			}

			'!' => {                                                                                		//Logical Negation
				chars.next();
				if chars.peek() == Some(&'=') {
					chars.next();
					tokens.push(Token { kind: TokenKind::NotEq, value: String::from("!=") });
				} else {
					tokens.push(Token { kind: TokenKind::Not, value: String::from("!") });
				}
			}

			'<' => {
				chars.next();
				if chars.peek() == Some(&'=') {
					chars.next();
					tokens.push(Token { kind: TokenKind::LtEq, value: String::from("<=") });
				} else {
					tokens.push(Token { kind: TokenKind::Lt, value: String::from("<") });
				}
			}

			'>' => {
				chars.next();
				if chars.peek() == Some(&'=') {
					chars.next();
					tokens.push(Token { kind: TokenKind::GtEq, value: String::from(">=") });
				} else {
					tokens.push(Token { kind: TokenKind::Gt, value: String::from(">") });
				}
			}

			'&' => {
				chars.next();
				if chars.peek() == Some(&'&') {
					chars.next();
					tokens.push(Token { kind: TokenKind::And, value: String::from("&&") });
				} else {
					return Err(String::from("Expected '&&'"));
				}
			}

			'|' => {
				chars.next();
				if chars.peek() == Some(&'|') {
					chars.next();
					tokens.push(Token { kind: TokenKind::Or, value: String::from("||") });
				} else {
					return Err(String::from("Expected '||'"));
				}
			}

			'+' => { chars.next(); tokens.push(Token { kind: TokenKind::Plus,   value: String::from("+") }); }
			'-' => { chars.next(); tokens.push(Token { kind: TokenKind::Minus,  value: String::from("-") }); }
			'*' => { chars.next(); tokens.push(Token { kind: TokenKind::Star,   value: String::from("*") }); }
			'/' => { chars.next(); tokens.push(Token { kind: TokenKind::Slash,  value: String::from("/") }); }
			'(' => { chars.next(); tokens.push(Token { kind: TokenKind::LParen, value: String::from("(") }); }
			')' => { chars.next(); tokens.push(Token { kind: TokenKind::RParen, value: String::from(")") }); }
			'{' => { chars.next(); tokens.push(Token { kind: TokenKind::LBrace, value: String::from("{") }); }
			'}' => { chars.next(); tokens.push(Token { kind: TokenKind::RBrace, value: String::from("}") }); }

			other => return Err(format!("Unexpected character: {}", other)),
		}
	}

	Ok(tokens)
}

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

	fn parse_primary(&mut self) -> String {																	//Parses a single primary: literal, variable, unary op, or grouped expression
		match self.peek().map(|t| t.kind.clone()) {
			Some(TokenKind::LParen) => {
				self.consume();
				let expr = self.parse_expr(0);
				self.consume();
				format!("({})", expr)
			}
			Some(TokenKind::Not) => {
				self.consume();
				let operand = self.parse_primary();
				format!("!{}", operand)
			}
			Some(TokenKind::Minus) => {
				self.consume();
				let operand = self.parse_primary();
				format!("-{}", operand)
			}
			Some(TokenKind::Id) => {																		//Adds the $ before a variable
				let t = self.consume().unwrap();
				format!("${}", t.value)
			}
			_ => {
				self.consume().map(|t| t.value).unwrap_or_default()
			}
		}
	}

	fn op_precedence(kind: &TokenKind) -> Option<u8> {														//Basic order of operations
		match kind {
			TokenKind::Or                                           				=> Some(1),
			TokenKind::And                                          				=> Some(2),
			TokenKind::Eq | TokenKind::NotEq                       					=> Some(3),
			TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq		=> Some(4),
			TokenKind::Plus | TokenKind::Minus                     					=> Some(5),
			TokenKind::Star | TokenKind::Slash                     					=> Some(6),
			_                                                       				=> None,
		}
	}

	fn parse_expr(&mut self, min_prec: u8) -> String {
		let mut left = self.parse_primary();

		loop {
			let prec = match self.peek() {
				Some(t) => match Self::op_precedence(&t.kind) {
					Some(p) if p >= min_prec => p,
					_ => break,
				},
				None => break,
			};

			let op = self.consume().unwrap().value;
			let right = self.parse_expr(prec + 1);
			left = format!("{} {} {}", left, op, right);
		}

		left
	}

	fn parse_block(&mut self) -> String {																	// Parses { ... } and returns the inner statements indented
		self.consume(); // LBrace
		let mut body = String::new();
		while let Some(t) = self.peek() {
			if t.kind == TokenKind::RBrace { break; }
			let stmt = self.statement();
			if !stmt.is_empty() {
				body.push_str(&format!("\t{};\n", stmt));
			}
		}
		self.consume(); // RBrace
		body
	}

	fn statement(&mut self) -> String {																		//for now, only two statements, Let and Print
		match self.peek().map(|t| t.kind.clone()) {
			Some(TokenKind::Let) => {																		//expects an ID token, an Assign token and an Expression
				self.consume();
				let var_name = self.consume().map(|t| t.value).unwrap_or_default();
				self.consume();
				let expr = self.parse_expr(0);
				format!("${} = {}", var_name, expr)
			}
			Some(TokenKind::Print) => {																		//prints the expression after it
				self.consume();
				let expr = self.parse_expr(0);
				format!("echo {}", expr)
			}
			Some(TokenKind::If) => {																		// if (cond) { ... } else { ... }
				self.consume(); // if
				self.consume(); // LParen
				let cond = self.parse_expr(0);
				self.consume(); // RParen
				let if_body = self.parse_block();
				let else_part = if self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Else) {
					self.consume(); // else
					let else_body = self.parse_block();
					format!(" else {{\n{}}}", else_body)
				} else {
					String::new()
				};
				return format!("if ({}) {{\n{}}}{}", cond, if_body, else_part);
			}
			_ => {
				self.consume();
				String::new()
			}
		}
	}

	fn transpile(&mut self) -> String {
		let mut output = String::from("<?php\n\n");												//starts PHP
		while self.pos < self.tokens.len() {
			let stmt = self.statement();														//calls statement() until there's no more tokens left
			if !stmt.is_empty() {
				output.push_str(&stmt);
				if !stmt.starts_with("if") {
					output.push_str(";\n");
				} else {
					output.push('\n');
				}
			}
		}
		output
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 || args[2] == "help" {
		eprintln!("Usage: ./PHPlus <filename.ez>");
		return;
	}

	let filename = &args[1];

	let code = match fs::read_to_string(filename) {
		Ok(c) => c,
		Err(_) => { eprintln!("Error: File '{}' not found.", filename); return; }
	};

	let tokens = match lexer(&code) {
		Ok(t) => t,
		Err(e) => { eprintln!("Lexer error: {}", e); return; }
	};

	let mut transpiler = Transpiler::new(tokens);
	let php_result = transpiler.transpile();

	let output_filename = filename.replace(".ez", ".php");
	match fs::write(&output_filename, &php_result) {
		Ok(_) => println!("Successfully compiled {} to {}", filename, output_filename),
		Err(e) => eprintln!("Failed to write output: {}", e),
	}
}
