use std::collections::{HashMap, HashSet};

use clap::Args;
use tracing::{debug, info, trace, warn};

#[derive(Args, Debug)]
#[command(
	name = "path-var",
	about = "Enhance httpie with path variable support",
	long_about = r#"
Enhance httpie with path variable support, allowing you to use path variables in URLs
and replace them with values from command line arguments.

Examples:
  # Replace :id with 123 in the URL
  httpie-oapi path-var -- http :8080/users/:id/posts :id=123

  # Multiple path variables
  httpie-oapi path-var -- http :8080/users/:userId/posts/:postId :userId=123 :postId=456

  # With other httpie options
  httpie-oapi path-var -- http :8080/api/v1/users/:id -v :id=123 --json --offline
"#
)]
pub(super) struct PathVarCommand {
	/// Raw command line arguments
	#[arg(raw = true)]
	args: Vec<String>,
}

impl PathVarCommand {
	/// Process the command line and execute the path variable replacement
	pub(super) fn run(&self) -> anyhow::Result<()> {
		info!("Processing command line: {:?}", self.args);
		let result = self.process_command_line();
		Self::write_result(&result);
		info!("Command processed successfully");
		Ok(())
	}

	/// Process the command line and return the processed arguments
	fn process_command_line(&self) -> Vec<String> {
		let mut args = self.args.clone();
		if args.is_empty() {
			debug!("Empty command line, returning as is");
			return args;
		}

		// Find the URL (first argument that matches URL patterns)
		let url_index = args.iter().position(|arg| Self::is_url_like(arg));
		trace!("URL search result: {:?}", url_index);

		let Some(url_index) = url_index else {
			debug!("No URL found in command line, returning as is");
			return args;
		};

		let url = &args[url_index];
		debug!("Found URL at index {}: {}", url_index, url);
		
		// Extract path variables from URL
		let path_vars = Self::extract_path_vars(url);
		debug!("Extracted path variables: {:?}", path_vars);

		if path_vars.is_empty() {
			debug!("No path variables found in URL, returning as is");
			return args;
		}

		// Process path variable assignments
		let (var_values, remaining_args) =
			Self::process_var_assignments(&args[url_index + 1..], &path_vars);
		debug!("Path variable values: {:?}", var_values);
		debug!("Remaining arguments: {:?}", remaining_args);

		// Replace path variables in URL
		let processed_url = Self::replace_path_vars(url, &path_vars, &var_values);
		debug!("Processed URL: {}", processed_url);

		// Reconstruct the command
		args[url_index] = processed_url;
		let mut result = Vec::with_capacity(url_index + 1 + remaining_args.len());
		result.extend(args[..=url_index].iter().cloned());
		result.extend(remaining_args);
		debug!("Final command: {:?}", result);
		result
	}

	/// Check if a string is a valid URL or URL-like string
	///
	/// This function recognizes:
	/// - Full URLs (http://, https://)
	/// - Domain:port format (e.g., localhost:8080)
	/// - Port-only format (e.g., :8080)
	/// - URLs with path components (e.g., :8080/users/:id/orders)
	///
	/// The function splits the input into two parts:
	/// 1. The host part (before the first '/')
	/// 2. The path part (after the first '/')
	///
	/// The host part must be a valid URL-like string (domain:port or port-only),
	/// while the path part can contain any characters.
	fn is_url_like(s: &str) -> bool {
		// Check for full URLs
		if s.starts_with("http://") || s.starts_with("https://") {
			debug!("String is a full URL: {}", s);
			return true;
		}

		// Split into host and path parts
		let Some((host, _)) = s.split_once('/') else {
            return false;
        };

		// Check if host part is a valid URL-like string
		let is_valid = host.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == ':');
		if is_valid {
			debug!("Host part is valid URL-like string: {}", host);
		} else {
			debug!("Host part is not a valid URL-like string: {}", host);
		}
		is_valid
	}

	/// Extract path variables from a URL
	///
	/// Returns a HashSet of path variables found in the URL.
	/// A path variable is a string that starts with ':' followed by
	/// multiple letters, numbers, or underscores
	/// eg :id, :postId, :id123, :id_123, :_id, :id_123_456
	///
	/// # Examples
	/// ```
	/// use httpie_oapi::command::path_var::PathVarCommand;
	/// 
	/// let vars = PathVarCommand::extract_path_vars("/users/:id/posts/:postId");
	/// assert_eq!(vars.len(), 2);
	/// assert!(vars.contains(":id"));
	/// assert!(vars.contains(":postId"));
	/// ```
	fn extract_path_vars(url: &str) -> HashSet<String> {
		trace!("Extracting path variables from URL: {}", url);
		let vars: HashSet<_> = url.split('/')
			.filter(|s| s.starts_with(':'))
			.filter(|s| s.len() > 1)
			.map(|s| s.to_string())
			.collect();
		debug!("Found path variables: {:?}", vars);
		vars
	}

	/// Process path variable assignments from command line arguments
	///
	/// Returns a tuple containing:
	/// - A HashMap of variable names to their values
	/// - A Vec of remaining arguments that are not path variable assignments
	fn process_var_assignments(
		args: &[String],
		path_vars: &HashSet<String>,
	) -> (HashMap<String, String>, Vec<String>) {
		trace!("Processing variable assignments from args: {:?}", args);
		trace!("Looking for variables: {:?}", path_vars);

		let mut var_values = HashMap::new();
		let mut remaining_args = Vec::new();

		for arg in args {
			if let Some((var_name, value)) = arg.split_once('=') {
				if var_name.starts_with(':') && path_vars.contains(var_name) {
					debug!("Found variable assignment: {} = {}", var_name, value);
					var_values.insert(var_name.to_string(), value.to_string());
					continue;
				}
			}
			trace!("Argument is not a variable assignment: {}", arg);
			remaining_args.push(arg.clone());
		}

		debug!("Processed variable assignments: {:?}", var_values);
		debug!("Remaining arguments: {:?}", remaining_args);
		(var_values, remaining_args)
	}

	/// Replace path variables in URL with their values
	fn replace_path_vars(
		url: &str,
		path_vars: &HashSet<String>,
		var_values: &HashMap<String, String>,
	) -> String {
		trace!("Replacing variables in URL: {}", url);
		trace!("Variables to replace: {:?}", path_vars);
		trace!("Variable values: {:?}", var_values);

		let mut result = url.to_string();
		for var in path_vars {
			if let Some(value) = var_values.get(var.as_str()) {
				debug!("Replacing {} with {}", var, value);
				result = result.replace(var, value);
			} else {
				warn!("No value found for variable: {}", var);
			}
		}
		debug!("URL after replacement: {}", result);
		result
	}

	/// Write the result to stdout
	fn write_result(result: &[String]) {
		debug!("Writing result to stdout: {:?}", result);
		println!("{}", result.join(" "));
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract_path_vars() {
		let test_cases = vec![
			("/users/:id/posts/:postId", vec![":id", ":postId"]),
			("/users/:id/posts/:postId/comments/:commentId", vec![":id", ":postId", ":commentId"]),
			("/users/:id123", vec![":id123"]),
			("/users/:id_123", vec![":id_123"]),
			("/users/123", vec![]),
			("/users/:", vec![]),
			("/users/:123", vec![":123"]),
			("/users/:_id", vec![":_id"]),
		];

		for (input, expected) in test_cases {
			let vars = PathVarCommand::extract_path_vars(input);
			assert_eq!(vars.len(), expected.len(), "Failed for input: {}", input);
			for var in expected {
				assert!(vars.contains(var), "Expected {} in vars for input: {}", var, input);
			}
		}
	}

	#[test]
	fn test_process_var_assignments() {
		let path_vars: HashSet<_> = vec![":id", ":postId"].into_iter().map(String::from).collect();
		let args = vec![
			":id=123".to_string(),
			"-v".to_string(),
			":postId=456".to_string(),
			"--json".to_string(),
			":unknown=789".to_string(),
		];

		let (var_values, remaining) = PathVarCommand::process_var_assignments(&args, &path_vars);

		assert_eq!(var_values.len(), 2);
		assert_eq!(var_values.get(":id"), Some(&"123".to_string()));
		assert_eq!(var_values.get(":postId"), Some(&"456".to_string()));
		assert_eq!(remaining, vec!["-v", "--json", ":unknown=789"]);
	}

	#[test]
	fn test_replace_path_vars() {
		let path_vars: HashSet<_> = vec![":id", ":postId"].into_iter().map(String::from).collect();
		let mut var_values = HashMap::new();
		var_values.insert(":id".to_string(), "123".to_string());
		var_values.insert(":postId".to_string(), "456".to_string());

		let url = "/users/:id/posts/:postId";
		let result = PathVarCommand::replace_path_vars(url, &path_vars, &var_values);
		assert_eq!(result, "/users/123/posts/456");

		// Test with missing value
		let mut var_values = HashMap::new();
		var_values.insert(":id".to_string(), "123".to_string());
		let result = PathVarCommand::replace_path_vars(url, &path_vars, &var_values);
		assert_eq!(result, "/users/123/posts/:postId");
	}

	#[test]
	fn test_is_url_like() {
		let valid_urls = vec![
			"http://example.com",
			"https://example.com",
			"localhost:8080",
			":8080",
			"example.com:8080",
			"example.com:8080/users",
			":8080/users/:id",
			"localhost:8080/users/:id/orders",
			"example.com:8080/api/v1/users/:id",
		];

		let invalid_urls = vec![
            "http",
            ":80=22",
            ":a=32",
            "a=3",
            "a=32",
            "foo==bar",
            "Authorization: Bearer 123",
			"not a url",
		];

		for url in valid_urls {
			assert!(PathVarCommand::is_url_like(url), "Should be valid URL: {}", url);
		}

		for url in invalid_urls {
			assert!(!PathVarCommand::is_url_like(url), "Should be invalid URL: {}", url);
		}
	}
}
