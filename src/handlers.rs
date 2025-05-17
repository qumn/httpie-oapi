use crate::{cli::{CompleteArgs, CompletionsArgs, ListApiArgs, ParamArgs, PathArgs, RefreshApiArgs, RemoveApiArgs, SaveApiArgs, Shell}, command_tokens::CommandLineTokens, config::Config, fish, openapi::ApiSpec};

pub fn handle_path_command(args: &PathArgs) {
	let config = Config::load();
	let api = config.get_api(&args.name).unwrap_or_else(|| {
		eprintln!("API '{}' not found", args.name);
		std::process::exit(1);
	});

	let endpoints = api.get_endpoints();
	let filtered: Vec<_> =
		if let Some(pattern) = &args.pattern { endpoints.filter(pattern) } else { endpoints.all() };

	for endpoint in filtered {
		if args.fish {
			println!("{}{}", api.base_url, endpoint.fish_complete_format());
		} else {
			println!("{}{}", api.base_url, endpoint.fzf_list_format());
		}
	}
}

pub fn handle_param_command(args: &ParamArgs) {
	let config = Config::load();
	let api = config.get_api(&args.name).unwrap_or_else(|| {
		eprintln!("API '{}' not found", args.name);
		std::process::exit(1);
	});

	let endpoints = api.get_endpoints();
	let endpoint = endpoints.find(&args.path);

	if let Some(ep) = endpoint {
		let filtered_params: Vec<_> = if let Some(pat) = &args.pattern {
			ep.params.iter().filter(|param| param.name.contains(pat)).collect()
		} else {
			ep.params.iter().collect()
		};

		for param in filtered_params {
			if args.fish {
				println!("{}", param);
			} else {
				println!("{}", param.fish_complete_format());
			}
		}
	} else {
		eprintln!("No endpoint matched path '{}'", args.path);
		std::process::exit(1);
	}
}

/// Handle command line completion with smart suggestions based on context
///
/// The completion logic follows these rules:
/// 1. When no base_url is matched, show all available API specs
/// 2. When cursor is on a base_url token, show all paths for that API spec
/// 3. When base_url and path are matched, show all parameters for that path
///
/// # Arguments
///
/// * `args` - The completion arguments containing the command line and cursor position
pub fn handle_complete(args: &CompleteArgs) {
	tracing::info!("Processing completion request: line={}, cursor_pos={}", args.line, args.cursor_pos);
	let config = Config::load();
	let tokens = CommandLineTokens::new(&args.line, args.cursor_pos);
	let apis = config.list_apis();
	tracing::debug!("Parsed tokens: {:?}", tokens);

	let (mut matched_api, mut matched_token) = (None, None);
	// first check if the base_url is matched
	for &api in apis.iter() {
		tracing::debug!("Checking API: {}", api.name);
		if let Some(token) = tokens.find_token_starting_with(&api.base_url) {
			tracing::info!("Found matching API: {}, token: {}", api.name, token.text);
			matched_api = Some(api);
			matched_token = Some(token);
		}
	}

	if matched_api.is_none() {
		tracing::info!("No matching API found, displaying all APIs");
		for api in apis {
			println!("{}\t{}", api.base_url, api.name);
		}
		return;
	}
	let matched_api = matched_api.unwrap();
	let matched_token = matched_token.unwrap();

	// If cursor is on base_url token, display all paths
	if let Some(current_token) = tokens.current_token() {
		tracing::debug!("Current token at cursor: {:?}", current_token);
		if current_token.text.starts_with(&matched_api.base_url) {
			tracing::info!("Cursor is on base_url token, displaying all paths");
			for ep in matched_api.get_endpoints().all() {
				println!("{}{}", matched_api.base_url, ep.fish_complete_format());
			}
			return;
		}
	}

	let path = matched_token.text.strip_prefix(&matched_api.base_url).unwrap_or(&matched_token.text);
	tracing::info!("Attempting to match path: {}", path);
	for ep in matched_api.get_endpoints().filter(path) {
		tracing::info!("Found matching path: {}", ep.path);
		// Get all parameters and filter out those that start with any existing token
		tracing::debug!("{:?}", tokens);
		for param in ep.get_params_sort() {
			tracing::debug!("find a param: {}" ,param.name);
			if !tokens.has_token_starting_with(&param.name) {
				println!("{}", param.fish_complete_format());
			}
		}
		return;
	}
	tracing::warn!("No matching path found");
}

pub fn handle_completions(args: &CompletionsArgs) {
	match args.shell {
		Shell::Fish => {
			if let Err(e) = fish::generate_completion(args.output.clone()) {
				eprintln!("Failed to generate fish completion: {}", e);
				std::process::exit(1);
			}
		}
	}
}

pub fn handle_save_api(args: &SaveApiArgs) {
	let mut config = Config::load();

	// Check if API already exists
	if !args.force && config.get_api(&args.name).is_some() {
		eprintln!("Error: API '{}' already exists. Use --force to overwrite.", args.name);
		std::process::exit(1);
	}

	let api = ApiSpec::new(args.name.clone(), args.url.clone(), args.base_url.clone());

	// Force download and cache endpoints
	api.refresh_endpoints_cache();

	config.add_api(args.name.clone(), args.url.clone(), args.base_url.clone());
	config.save();

	if args.force {
		println!("Updated API '{}' successfully", args.name);
	} else {
		println!("Added API '{}' successfully", args.name);
	}
}

pub fn handle_refresh_api(args: &RefreshApiArgs) {
	let config = Config::load();

	let names_to_refresh = if args.names.is_empty() {
		// If no names provided, get all API names
		config.list_apis().iter().map(|&api| api.name.to_string()).collect()
	} else {
		args.names.clone()
	};

	for name in &names_to_refresh {
		match config.get_api(name) {
			Some(api) => {
				api.refresh_endpoints_cache();
				println!("Refreshed cache for API '{}' successfully", name);
			}
			None => {
				eprintln!("Warning: API '{}' not found, skipping", name);
			}
		}
	}

	config.save();
}

pub fn handle_remove_api(args: &RemoveApiArgs) {
	let mut config = Config::load();
	if config.remove_api(&args.name) {
		config.save();
		println!("Removed API '{}' successfully", args.name);
	} else {
		eprintln!("API '{}' not found", args.name);
		std::process::exit(1);
	}
}

pub fn handle_list_apis(args: &ListApiArgs) {
	let config = Config::load();
	let apis = config.list_apis();

	if apis.is_empty() {
		println!("No APIs registered");
		return;
	}

	for api in apis {
		if args.detailed {
			println!("Name: {}", api.name);
			println!("URL: {}", api.url);
			println!("Base URL: {}", api.base_url);
			println!("Cache: {}", Config::get_cache_path(&api.name).display());
			println!();
		} else {
			println!("{}\t{}", api.name, api.url);
		}
	}
}
