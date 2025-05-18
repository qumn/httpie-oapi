#[derive(Debug)]
pub struct Tokens {
	tokens: Vec<Token>,
	cursor_pos: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
	pub text: String,
	pub start: usize,
	pub end: usize,
}

impl Tokens {
	pub fn new(line: &str, cursor_pos: usize) -> Self {
		let mut tokens = Vec::new();
		let mut current_pos = 0;

		// Split command line using shell_words
		for token in shell_words::split(line).unwrap_or_default() {
			let start = line[current_pos..].find(&token).unwrap_or(0) + current_pos;
			let end = start + token.len();
			tokens.push(Token { text: token, start, end });
			current_pos = end;
		}
		Self { tokens, cursor_pos }
	}

	/// Get the token at the current cursor position
	///
	/// # Returns
	///
	/// Returns a reference to the token that contains the cursor position,
	/// or None if no token contains the cursor position
	pub fn current_token(&self) -> Option<&Token> {
		self.tokens.iter().find(|token| self.cursor_pos >= token.start && self.cursor_pos <= token.end)
	}

	/// Find a token that starts with the given base_url
	///
	/// # Arguments
	///
	/// * `base_url` - The base_url to check
	///
	/// # Returns
	///
	/// Returns a reference to the first token that starts with the base_url,
	/// or None if no token starts with the base_url
	pub fn find_token_starting_with(&self, prefix: &str) -> Option<&Token> {
		self.tokens.iter().find(|token| token.text.starts_with(prefix))
	}

	pub fn has_token_starting_with(&self, text: &str) -> bool {
		self.tokens.iter().any(|t| t.text.starts_with(text))
	}
}

#[cfg(test)]
mod tests {
	/// # Examples
	///
	/// ```
	/// let tokens = tokens!("http example.com|"); // cursor at the end
	/// let tokens = tokens!("http |example.com"); // cursor between tokens
	/// let tokens = tokens!("http ex|ample.com"); // cursor in the middle of a token
	/// ```
	#[macro_export]
	macro_rules! tokens {
		($line:literal) => {{
			let line = $line;
			let cursor_pos = line.find('|').expect("No cursor position marker '|' found");
			let line = line.replace('|', "");
			super::Tokens::new(&line, cursor_pos)
		}};
	}

	#[test]
	fn test_current_token_at_start() {
		let tokens = tokens!("|http example.com");
		let token = tokens.current_token().unwrap();
		assert_eq!(token.text, "http");
	}

	#[test]
	fn test_current_token_at_middle() {
		let tokens = tokens!("http ex|ample.com");
		let token = tokens.current_token().unwrap();
		assert_eq!(token.text, "example.com");
	}

	#[test]
	fn test_current_token_at_end() {
		let tokens = tokens!("http example.com|");
		let token = tokens.current_token().unwrap();
		assert_eq!(token.text, "example.com");
	}

	#[test]
	fn test_current_token_between_tokens() {
		let tokens = tokens!("http |example.com");
		let token = tokens.current_token().unwrap();
		assert_eq!(token.text, "example.com");
	}

	#[test]
	fn test_find_token_starting_with() {
		let tokens = tokens!("http http://api1.com /users|");
		let token = tokens.find_token_starting_with("http://api1.com").unwrap();
		assert_eq!(token.text, "http://api1.com");
		assert!(tokens.find_token_starting_with("http://api2.com").is_none());
	}

	#[test]
	fn test_find_token_starting_with_partial_match() {
		let tokens = tokens!("http http://api1.com/v1|");
		let token = tokens.find_token_starting_with("http://api1.com").unwrap();
		assert_eq!(token.text, "http://api1.com/v1");
	}

	#[test]
	fn test_find_token_starting_with_not_found() {
		let tokens = tokens!("http example.com|");
		assert!(tokens.find_token_starting_with("http://api1.com").is_none());
	}
}
