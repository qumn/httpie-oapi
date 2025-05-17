use crate::cli::{
	CompleteArgs, CompletionsArgs, ListApiArgs, ParamArgs, PathArgs, RefreshApiArgs, RemoveApiArgs,
	SaveApiArgs, Shell,
};
use crate::config::Config;
use crate::fish;
use crate::openapi::ApiSpec;

pub fn handle_path_command(args: &PathArgs) {
	let mut config = Config::load();
	let api = config.get_api_mut(&args.name).unwrap_or_else(|| {
		eprintln!("API '{}' not found", args.name);
		std::process::exit(1);
	});

	let endpoints = api.get_endpoints();
	let filtered: Vec<_> =
		if let Some(pattern) = &args.pattern { endpoints.filter(pattern) } else { endpoints.all() };

	for endpoint in filtered {
		if args.fish {
			println!("{}", endpoint.fish_complete_format());
		} else {
			println!("{}", endpoint.fzf_list_format());
		}
	}
}

pub fn handle_param_command(args: &ParamArgs) {
	let mut config = Config::load();
	let api = config.get_api_mut(&args.name).unwrap_or_else(|| {
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

pub fn handle_complete(args: &CompleteArgs) {
	let mut config = Config::load();
	let api = config.get_api_mut(&args.name).unwrap_or_else(|| {
		eprintln!("API '{}' not found", args.name);
		std::process::exit(1);
	});

	let endpoints = api.get_endpoints();
	let tokens = shell_words::split(&args.line).unwrap_or_default();
	let path_token = tokens.get(1).map(|s| s.as_str()).unwrap_or("");

	let is_cursor_in_path = {
		if let Some(pos) = args.line.find(path_token) {
			let end_pos = pos + path_token.len();
			args.cursor_pos >= pos && args.cursor_pos <= end_pos
		} else {
			false
		}
	};

	if is_cursor_in_path {
		for ep in &endpoints.all() {
			if ep.path.starts_with(path_token) {
				println!("{}", ep.fish_complete_format());
			}
		}
	} else {
		let eps = endpoints.filter(path_token);
		eps.iter().flat_map(|ep| ep.get_params_sort()).for_each(|param| {
			println!("{}", param.fish_complete_format());
		});
	}
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
	let mut config = Config::load();

	let names_to_refresh = if args.names.is_empty() {
		// If no names provided, get all API names
		config.list_apis().iter().map(|(name, _)| name.to_string()).collect()
	} else {
		args.names.clone()
	};

	for name in &names_to_refresh {
		match config.get_api_mut(name) {
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

	for (name, api) in apis {
		if args.detailed {
			println!("Name: {}", name);
			println!("URL: {}", api.url);
			println!("Base URL: {}", api.base_url);
			println!("Cache: {}", Config::get_cache_path(name).display());
			println!();
		} else {
			println!("{}\t{}", name, api.url);
		}
	}
}
