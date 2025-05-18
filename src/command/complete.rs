use clap::Args;

use crate::{config::Config, tokens::Tokens};

#[derive(Args, Debug)]
pub(super) struct CompleteCommand {
	/// The current command line input to complete
	/// This is the full command line string up to the cursor position
	/// Example: "http https://api.example.com/users"
	#[arg(long, short = 'l', value_name = "LINE")]
	line: String,

	/// The current cursor position in the command line
	/// This is used to determine which token is being completed
	/// Example: if LINE is "http https://api.example.com/users" and CURSOR_POS is 25,
	/// the cursor is at "users"
	#[arg(long, short = 'c', value_name = "CURSOR_POS")]
	cursor_pos: usize,
}

impl CompleteCommand {
	/// Handle command line completion with smart suggestions based on context
	///
	/// This command is used internally by the fish shell completion system to provide
	/// intelligent command line completion for HTTPie commands. It analyzes the current
	/// command line context and suggests appropriate completions based on the following rules:
	///
	/// 1. If no token contains any base_url, show all available API specs
	///    Example: "http " -> shows all registered API base URLs
	///
	/// 2. If a token contains a base_url, use that API spec
	///    Example: "http https://api.example.com" -> uses api.example.com's spec
	///
	/// 3. If cursor is on the base_url token, show all paths for that API
	///    Example: "http https://api.example.com" -> shows all available endpoints
	///
	/// 4. If cursor is not on base_url token, show all parameters for the matched path
	///    Example: "http https://api.example.com/users " -> shows all parameters for /users
	///
	/// The completion suggestions are formatted for fish shell, with descriptions
	/// and proper parameter formatting (e.g., query parameters with ==, headers with :).
	pub(super) fn run(&self, config: &Config) -> anyhow::Result<()> {
		tracing::info!(
			"Processing completion request: line={}, cursor_pos={}",
			self.line,
			self.cursor_pos
		);
		let tokens = Tokens::new(&self.line, self.cursor_pos);
		let apis = config.list_apis();
		tracing::debug!("Parsed tokens: {:?}", tokens);

		// Step 1: Check if any token contains a base_url
		let mut matched_api = None;
		let mut matched_token = None;

		for &api in apis.iter() {
			if let Some(token) = tokens.find_token_starting_with(&api.base_url) {
				matched_api = Some(api);
				matched_token = Some(token);
				break;
			}
		}

		// If no base_url is found in any token, show all API specs
		let (Some(matched_api), Some(matched_token)) = (matched_api, matched_token) else {
			tracing::info!("No base_url found in tokens, showing all API specs");
			for api in apis {
				println!("{}/\t{}", api.base_url, api.name);
			}
			return Ok(());
		};

		// Step 2 & 3: Check if cursor is on the base_url token
		if let Some(current_token) = tokens.current_token() {
			if current_token.text.starts_with(&matched_api.base_url) {
				tracing::info!("Cursor is on base_url token, showing all paths");
				for ep in matched_api.get_endpoints().all() {
					println!("{}", ep.fish_complete_format(&matched_api.base_url));
				}
				return Ok(());
			}
		}

		// Step 4: Show parameters for the matched path
		let path =
			matched_token.text.strip_prefix(&matched_api.base_url).unwrap_or(&matched_token.text);
		tracing::info!("Looking for parameters for path: {}", path);

		for ep in matched_api.get_endpoints().filter(path) {
			tracing::info!("Found matching endpoint: {}", ep.path);
			for param in ep.get_params_sort() {
				if !tokens.has_token_starting_with(&param.name) {
					println!("{}", param.fish_complete_format());
				}
			}
		}

		Ok(())
	}
}
