use std::env;
use std::fs;
use std::fmt;

// --- ERROR TYPE ---

/// Represents a compiler error with a human-readable message and the source location where it occurred.
#[derive(Debug)]
struct LangError {
	message: String,
	line: usize,
	col: usize,
}

impl LangError {
	/// Creates a new `LangError` with the given message, line number, and column number.
	fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
		LangError { message: message.into(), line, col }
	}
}

impl fmt::Display for LangError {
	/// Formats the error as `[line:col] Error: message` for printing to stderr.
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
	While,          // while keyword
	For,            // for keyword
	Class,          // class keyword
	New,            // new keyword
	Fn,             // fn keyword
	Semicolon,      // ; used as separator in for loops

	// data types
	String,
	Number,

	Id,     // variable name
	Self_,  // self keyword, becomes $this in PHP
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
	True,   // true
	False,  // false
	Dot,    // . property access
	Comma,  // , parameter separator
	Private, // private keyword
}

/// A single lexical unit produced by the lexer.
/// Stores the token's kind, its raw source string, and the source location it came from.
#[derive(Debug, Clone)]
struct Token {  //Pairs the data type and the value from source code
	kind: TokenKind,
	value: String,
	line: usize,
	col: usize,
}

/// Converts raw EzLang source code into a flat list of tokens.
///
/// Walks the source string character by character, classifying each chunk into
/// a `Token` with kind, value, and source location. Whitespace is skipped.
///
/// # Errors
/// Returns a `LangError` on: unexpected characters, unterminated strings,
/// malformed numbers (multiple decimal points), or lone `&` / `|`.
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
					"print"   => TokenKind::Print,
					"let"     => TokenKind::Let,
					"if"      => TokenKind::If,
					"else"    => TokenKind::Else,
					"while"   => TokenKind::While,
					"for"     => TokenKind::For,
					"true"    => TokenKind::True,
					"false"   => TokenKind::False,
					"echo"    => TokenKind::Print,   // echo is an alias for print
					"class"   => TokenKind::Class,
					"new"     => TokenKind::New,
					"fn"      => TokenKind::Fn,
					"self"    => TokenKind::Self_,
					"private" => TokenKind::Private,
					_         => TokenKind::Id,
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

			'.' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Dot,       value: String::from("."), line, col: c }); }
			',' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Comma,     value: String::from(","), line, col: c }); }
			'+' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Plus,      value: String::from("+"), line, col: c }); }
			'-' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Minus,     value: String::from("-"), line, col: c }); }
			'*' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Star,      value: String::from("*"), line, col: c }); }
			'/' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Slash,     value: String::from("/"), line, col: c }); }
			'(' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::LParen,    value: String::from("("), line, col: c }); }
			')' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::RParen,    value: String::from(")"), line, col: c }); }
			'{' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::LBrace,    value: String::from("{"), line, col: c }); }
			'}' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::RBrace,    value: String::from("}"), line, col: c }); }
			';' => { let c = col; chars.next(); col += 1; tokens.push(Token { kind: TokenKind::Semicolon, value: String::from(";"), line, col: c }); }

			other => return Err(LangError::new(format!("Unexpected character '{}'", other), line, col)),
		}
	}

	Ok(tokens)
}

// --- TRANSPILER ---

/// Single-pass transpiler that consumes the token list and emits PHP source code.
/// No AST is built; tokens are consumed and PHP strings are emitted directly.
struct Transpiler {                                                                                 		//Owns the token list and tracks the current position
	tokens: Vec<Token>,
	pos: usize,
}

impl Transpiler {
	/// Creates a new `Transpiler` from a token list, with `pos` starting at 0.
	fn new(tokens: Vec<Token>) -> Self {
		Transpiler { tokens, pos: 0 }
	}

	/// Returns a reference to the current token without advancing `pos`.
	fn peek(&self) -> Option<&Token> {                                                              		//Returns a reference without moving the position
		self.tokens.get(self.pos)
	}

	/// Clones and returns the current token, then advances `pos` by one.
	/// Returns `None` if already past the end of the token list.
	fn consume(&mut self) -> Option<Token> {																//Returns a Clone of the current token and moves the counter
		let token = self.tokens.get(self.pos).cloned();
		self.pos += 1;
		token
	}

	/// Consumes the current token and returns it if it matches `kind`.
	/// If it doesn't match, returns a `LangError` describing what was expected vs. what was found.
	/// `context` is a short string appended to the error message to clarify where the token was expected.
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

	/// Parses the smallest indivisible unit of an expression.
	///
	/// Handles: parenthesised groups `(expr)`, unary `!expr`, unary `-expr`,
	/// variable references (prepends `$`), `self` (becomes `$this`),
	/// string literals, number literals, boolean literals,
	/// and `new ClassName()` instantiation.
	/// Called by `parse_expr` to get the left-hand side before looking for a binary operator.
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
			Some(TokenKind::Self_) => {																		// self.attr becomes $this->attr
				self.consume();
				self.parse_postfix(String::from("$this"))
			}
			Some(TokenKind::Id) => {																		//Adds the $ before a variable
				let t = self.consume().unwrap();
				if self.peek().map(|t| t.kind.clone()) == Some(TokenKind::LParen) {
					self.consume();
					let mut args: Vec<String> = Vec::new();
					while self.peek().map(|t| t.kind.clone()) != Some(TokenKind::RParen) {
						if !args.is_empty() {
							self.expect(TokenKind::Comma, "between function arguments")?;
						}
						args.push(self.parse_expr(0)?);
					}
					self.expect(TokenKind::RParen, "to close function call")?;
					let base = format!("{}({})", t.value, args.join(", "));
					self.parse_postfix(base)
				} else {
					let base = format!("${}", t.value);
					self.parse_postfix(base)
				}
			}
			Some(TokenKind::String) | Some(TokenKind::Number) => {
				Ok(self.consume().unwrap().value)
			}
			Some(TokenKind::True)  => { self.consume(); Ok(String::from("true"))  }
			Some(TokenKind::False) => { self.consume(); Ok(String::from("false")) }
			Some(TokenKind::New)   => {																		// new ClassName() instantiation
				let tok = self.consume().unwrap();
				let class_name = match self.consume() {
					Some(t) if t.kind == TokenKind::Id => t.value,
					Some(t) => return Err(LangError::new(
						format!("Expected class name after 'new', got '{}'", t.value),
						t.line, t.col,
					)),
					None => return Err(LangError::new("Expected class name after 'new'", tok.line, tok.col)),
				};
				self.expect(TokenKind::LParen, "after class name in 'new'")?;
				self.expect(TokenKind::RParen, "to close 'new' argument list")?;
				let base = format!("new {}()", class_name);
				self.parse_postfix(base)
			}
			Some(_) => {
				let t = self.peek().unwrap();
				Err(LangError::new(
					format!("Unexpected token '{}' in expression", t.value),
					t.line, t.col,
				))
			}
			None => Err(LangError::new("Unexpected end of file in expression", 0, 0)),
		}
	}

	/// Parses a postfix property access chain: `expr.attr` or `expr.attr.attr2`.
	///
	/// Called after `parse_primary` to consume any number of `.attr` suffixes.
	/// Translates `obj.attr` to `$obj->attr` in PHP. Chains left-to-right.
	fn parse_postfix(&mut self, mut left: String) -> Result<String, LangError> {
		while self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Dot) {
			let dot_tok = self.consume().unwrap();
			let attr = match self.consume() {
				Some(t) if t.kind == TokenKind::Id => t.value,
				Some(t) => return Err(LangError::new(
					format!("Expected attribute name after '.', got '{}'", t.value),
					t.line, t.col,
				)),
				None => return Err(LangError::new("Expected attribute name after '.'", dot_tok.line, dot_tok.col)),
			};
			if self.peek().map(|t| t.kind.clone()) == Some(TokenKind::LParen) {
				self.consume();
				let mut args: Vec<String> = Vec::new();
				while self.peek().map(|t| t.kind.clone()) != Some(TokenKind::RParen) {
					if !args.is_empty() {
						self.expect(TokenKind::Comma, "between method arguments")?;
					}
					args.push(self.parse_expr(0)?);
				}
				self.expect(TokenKind::RParen, "to close method call")?;
				left = format!("{}->{}({})", left, attr, args.join(", "));
			} else {
				left = format!("{}->{}", left, attr);
			}
		}
		Ok(left)
	}

	/// Maps a binary operator token to its precedence level (1 = loosest, 6 = tightest).
	/// Returns `None` for any token that is not a binary operator,
	/// which signals `parse_expr` to stop climbing.
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

	/// Parses a binary expression using precedence climbing.
	///
	/// Calls `parse_primary` for the initial left-hand side, then loops consuming
	/// binary operators whose precedence is >= `min_prec`. Recursing with `prec + 1`
	/// enforces left-associativity. Pass `min_prec = 0` to parse a full expression.
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

	/// Parses a brace-delimited block `{ ... }` and returns its contents as indented PHP.
	///
	/// `depth` controls the indentation level: each statement inside is prefixed
	/// with `depth` tabs. Block statements (if/while/for/class) get a trailing newline
	/// instead of a semicolon. The surrounding braces are consumed but not included
	/// in the return value — the caller formats them.
	fn parse_block(&mut self, depth: usize) -> Result<String, LangError> {								// Parses { ... } and returns the inner statements indented
		let indent = "\t".repeat(depth);
		self.expect(TokenKind::LBrace, "to open block")?;
		let mut body = String::new();
		while let Some(t) = self.peek() {
			if t.kind == TokenKind::RBrace { break; }
			let stmt = self.statement(depth)?;
			if !stmt.is_empty() {
				let is_block_stmt = stmt.starts_with("if")
					|| stmt.starts_with("while")
					|| stmt.starts_with("for")
					|| stmt.starts_with("class")
					|| stmt.starts_with("function");
				if is_block_stmt {
					body.push_str(&format!("{}{}\n", indent, stmt));
				} else {
					body.push_str(&format!("{}{};\n", indent, stmt));
				}
			}
		}
		self.expect(TokenKind::RBrace, "to close block")?;
		Ok(body)
	}

	/// Parses and emits a single statement.
	///
	/// Dispatches on the current token kind:
	/// - `let x = expr`          → variable declaration
	/// - `x = expr`              → variable reassignment
	/// - `x.attr = expr`         → property assignment
	/// - `self.attr = expr`      → self property assignment (becomes `$this->attr`)
	/// - `print expr`            → echo statement
	/// - `class Name { ... }`    → class definition with properties and methods
	/// - `fn name(params) { ... }` → top-level function definition
	/// - `if / while / for`        → control flow (calls `parse_block` for bodies)
	///
	/// `depth` is passed through to `parse_block` for correct indentation.
	fn statement(&mut self, depth: usize) -> Result<String, LangError> {								//for now, only two statements, Let and Print
		match self.peek().map(|t| t.kind.clone()) {
			Some(TokenKind::Let) => {																	//expects an ID token, an Assign token and an Expression
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
			Some(TokenKind::Print) => {																	//prints the expression after it
				self.consume();
				let expr = self.parse_expr(0)?;
				Ok(format!("echo {}", expr))
			}
			Some(TokenKind::If) => {																	// if (cond) { ... } else { ... }
				self.consume();
				self.expect(TokenKind::LParen, "after 'if'")?;
				let cond = self.parse_expr(0)?;
				self.expect(TokenKind::RParen, "to close 'if' condition")?;
				let indent = "\t".repeat(depth);
				let if_body = self.parse_block(depth + 1)?;
				let else_part = if self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Else) {
					self.consume();
					let else_body = self.parse_block(depth + 1)?;
					format!(" else {{\n{}{}}}", else_body, indent)
				} else {
					String::new()
				};
				Ok(format!("if ({}) {{\n{}{}}}{}", cond, if_body, indent, else_part))
			}
			Some(TokenKind::While) => {																	// while (cond) { ... }
				self.consume();
				self.expect(TokenKind::LParen, "after 'while'")?;
				let cond = self.parse_expr(0)?;
				self.expect(TokenKind::RParen, "to close 'while' condition")?;
				let indent = "\t".repeat(depth);
				let body = self.parse_block(depth + 1)?;
				Ok(format!("while ({}) {{\n{}{}}}", cond, body, indent))
			}
			Some(TokenKind::For) => {																	// for (init; cond; step) { ... }
				self.consume();
				self.expect(TokenKind::LParen, "after 'for'")?;
				let init = self.parse_for_clause()?;
				self.expect(TokenKind::Semicolon, "after 'for' init")?;
				let cond = self.parse_expr(0)?;
				self.expect(TokenKind::Semicolon, "after 'for' condition")?;
				let step = self.parse_for_clause()?;
				self.expect(TokenKind::RParen, "to close 'for' header")?;
				let indent = "\t".repeat(depth);
				let body = self.parse_block(depth + 1)?;
				Ok(format!("for ({}; {}; {}) {{\n{}{}}}", init, cond, step, body, indent))
			}
			Some(TokenKind::Fn) => {																		// fn name(params) { ... } top-level function
				let fn_tok = self.consume().unwrap();
				let fn_name = match self.consume() {
					Some(t) if t.kind == TokenKind::Id => t.value,
					Some(t) => return Err(LangError::new(
						format!("Expected function name after 'fn', got '{}'", t.value),
						t.line, t.col,
					)),
					None => return Err(LangError::new("Expected function name after 'fn'", fn_tok.line, fn_tok.col)),
				};
				self.expect(TokenKind::LParen, "after function name")?;
				let mut params: Vec<String> = Vec::new();
				while self.peek().map(|t| t.kind.clone()) != Some(TokenKind::RParen) {
					if !params.is_empty() {
						self.expect(TokenKind::Comma, "between function parameters")?;
					}
					let param = match self.consume() {
						Some(t) if t.kind == TokenKind::Id => t.value,
						Some(t) => return Err(LangError::new(
							format!("Expected parameter name, got '{}'", t.value),
							t.line, t.col,
						)),
						None => return Err(LangError::new("Expected parameter name, got end of file", 0, 0)),
					};
					params.push(format!("${}", param));
				}
				self.expect(TokenKind::RParen, "to close function parameter list")?;
				let param_str = params.join(", ");
				let indent = "\t".repeat(depth);
				let body = self.parse_block(depth + 1)?;
				Ok(format!("function {}({}) {{\n{}{}}}", fn_name, param_str, body, indent))
			}
			Some(TokenKind::Class) => {																	// class Name { let prop = val ... fn method(params) { ... } }
				let class_tok = self.consume().unwrap();
				let class_name = match self.consume() {
					Some(t) if t.kind == TokenKind::Id => t.value,
					Some(t) => return Err(LangError::new(
						format!("Expected class name after 'class', got '{}'", t.value),
						t.line, t.col,
					)),
					None => return Err(LangError::new("Expected class name after 'class'", class_tok.line, class_tok.col)),
				};
				self.expect(TokenKind::LBrace, "to open class body")?;
				let mut body = String::new();
				while let Some(t) = self.peek() {
					if t.kind == TokenKind::RBrace { break; }
					match self.peek().map(|t| t.kind.clone()) {
						Some(TokenKind::Fn) => {														// method: fn name(params) { ... }
							self.consume();
							let method_tok = match self.consume() {
								Some(t) if t.kind == TokenKind::Id => t,
								Some(t) => return Err(LangError::new(
									format!("Expected method name after 'fn', got '{}'", t.value),
									t.line, t.col,
								)),
								None => return Err(LangError::new("Expected method name after 'fn'", 0, 0)),
							};
							self.expect(TokenKind::LParen, "after method name")?;
							let mut params: Vec<String> = Vec::new();
							while self.peek().map(|t| t.kind.clone()) != Some(TokenKind::RParen) {
								if !params.is_empty() {
									self.expect(TokenKind::Comma, "between method parameters")?;
								}
								let param = match self.consume() {
									Some(t) if t.kind == TokenKind::Id => t.value,
									Some(t) => return Err(LangError::new(
										format!("Expected parameter name, got '{}'", t.value),
										t.line, t.col,
									)),
									None => return Err(LangError::new("Expected parameter name, got end of file", 0, 0)),
								};
								params.push(format!("${}", param));
							}
							self.expect(TokenKind::RParen, "to close method parameter list")?;
							let param_str = params.join(", ");
							let method_body = self.parse_block(2)?;
							body.push_str(&format!("\tpublic function {}({}) {{\n{}\t}}\n", method_tok.value, param_str, method_body));
						}
						Some(TokenKind::Private) | Some(TokenKind::Let) => {						// property: [private] let name = val
							let visibility = if self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Private) {
								self.consume();
								"private"
							} else {
								"public"
							};
							self.expect(TokenKind::Let, "after visibility modifier")?;
							let prop_tok = match self.consume() {
								Some(t) if t.kind == TokenKind::Id => t,
								Some(t) => return Err(LangError::new(
									format!("Expected property name in class body, got '{}'", t.value),
									t.line, t.col,
								)),
								None => return Err(LangError::new("Expected property name in class body", 0, 0)),
							};
							self.expect(TokenKind::Assign, "after property name in class body")?;
							let val = self.parse_expr(0)?;
							body.push_str(&format!("\t{} ${} = {};\n", visibility, prop_tok.value, val));
						}
						_ => {
							let t = self.peek().unwrap();
							return Err(LangError::new(
								format!("Expected 'let' or 'fn' in class body, got '{}'", t.value),
								t.line, t.col,
							));
						}
					}
				}
				self.expect(TokenKind::RBrace, "to close class body")?;
				Ok(format!("class {} {{\n{}}}", class_name, body))
			}
			Some(TokenKind::Self_) | Some(TokenKind::Id) => {											// bare assignment, method call, or bare function call
				let name_tok = self.consume().unwrap();
				// bare function call: foo(args)
				if name_tok.kind == TokenKind::Id && self.peek().map(|t| t.kind.clone()) == Some(TokenKind::LParen) {
					self.consume();
					let mut args: Vec<String> = Vec::new();
					while self.peek().map(|t| t.kind.clone()) != Some(TokenKind::RParen) {
						if !args.is_empty() {
							self.expect(TokenKind::Comma, "between function arguments")?;
						}
						args.push(self.parse_expr(0)?);
					}
					self.expect(TokenKind::RParen, "to close function call")?;
					return Ok(format!("{}({})", name_tok.value, args.join(", ")));
				}
				let mut lhs = if name_tok.kind == TokenKind::Self_ {
					String::from("$this")
				} else {
					format!("${}", name_tok.value)
				};
				while self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Dot) {
					let dot_tok = self.consume().unwrap();
					let attr = match self.consume() {
						Some(t) if t.kind == TokenKind::Id => t.value,
						Some(t) => return Err(LangError::new(
							format!("Expected attribute name after '.', got '{}'", t.value),
							t.line, t.col,
						)),
						None => return Err(LangError::new("Expected attribute name after '.'", dot_tok.line, dot_tok.col)),
					};
					if self.peek().map(|t| t.kind.clone()) == Some(TokenKind::LParen) {
						self.consume();
						let mut args: Vec<String> = Vec::new();
						while self.peek().map(|t| t.kind.clone()) != Some(TokenKind::RParen) {
							if !args.is_empty() {
								self.expect(TokenKind::Comma, "between method arguments")?;
							}
							args.push(self.parse_expr(0)?);
						}
						self.expect(TokenKind::RParen, "to close method call")?;
						lhs = format!("{}->{}({})", lhs, attr, args.join(", "));
					} else {
						lhs = format!("{}->{}", lhs, attr);
					}
				}
				if self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Assign) {
					self.consume();
					let expr = self.parse_expr(0)?;
					Ok(format!("{} = {}", lhs, expr))
				} else if lhs.contains("->") && lhs.ends_with(")") {
					Ok(lhs) // standalone method call statement
				} else {
					let t = self.peek();
					let (tline, tcol) = t.map(|t| (t.line, t.col)).unwrap_or((0, 0));
					let got = t.map(|t| t.value.clone()).unwrap_or_else(|| String::from("end of file"));
					Err(LangError::new(format!("Expected '=' in assignment, got '{}'", got), tline, tcol))
				}
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

	/// Parses the init or step clause inside a `for (init; cond; step)` header.
	///
	/// Looks one token ahead to distinguish three cases without consuming prematurely:
	/// - `let x = expr`  → fresh variable declaration
	/// - `x = expr`      → reassignment of an existing variable
	/// - anything else   → treated as a plain expression
	fn parse_for_clause(&mut self) -> Result<String, LangError> {										// Parses the init or step clause of a for loop (let assignment, bare assignment, or expression)
		let is_let = self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Let);
		let is_bare_assign = self.peek().map(|t| t.kind.clone()) == Some(TokenKind::Id)
			&& self.tokens.get(self.pos + 1).map(|t| t.kind.clone()) == Some(TokenKind::Assign);

		if is_let {
			self.consume();
			let name_tok = match self.consume() {
				Some(t) if t.kind == TokenKind::Id => t,
				Some(t) => return Err(LangError::new(
					format!("Expected variable name in 'for' clause, got '{}'", t.value),
					t.line, t.col,
				)),
				None => return Err(LangError::new("Expected variable name in 'for' clause, got end of file", 0, 0)),
			};
			self.expect(TokenKind::Assign, "in 'for' clause")?;
			let expr = self.parse_expr(0)?;
			Ok(format!("${} = {}", name_tok.value, expr))
		} else if is_bare_assign {
			let name_tok = self.consume().unwrap();
			self.consume();
			let expr = self.parse_expr(0)?;
			Ok(format!("${} = {}", name_tok.value, expr))
		} else {
			self.parse_expr(0)
		}
	}

	/// Drives the full transpilation pass and returns the complete PHP output string.
	///
	/// Prepends the PHP opening tag, then calls `statement` in a loop until all
	/// tokens are consumed. Block statements (if/while/for/class) get a trailing newline;
	/// all other statements get a semicolon and newline.
	fn transpile(&mut self) -> Result<String, LangError> {
		let mut output = String::from("<?php\n\n");												//starts PHP
		while self.pos < self.tokens.len() {
			let stmt = self.statement(0)?;														//calls statement() until there's no more tokens left
			if !stmt.is_empty() {
				let is_block_stmt = stmt.starts_with("if")
					|| stmt.starts_with("while")
					|| stmt.starts_with("for")
					|| stmt.starts_with("class")
					|| stmt.starts_with("function");
				output.push_str(&stmt);
				if is_block_stmt {
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

/// Entry point. Reads a `.ez` source file, runs the lexer and transpiler,
/// and writes the resulting PHP to a `.php` file with the same base name.
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
