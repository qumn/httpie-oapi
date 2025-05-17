use crate::cli::{CompleteArgs, CompletionsArgs, ParamArgs, PathArgs, Shell};
use crate::fish;
use crate::openapi::EndPoints;

pub fn handle_path_command(args: &PathArgs) {
	let endpoints = EndPoints::parse_json(&args.file);

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
	let endpoints = EndPoints::parse_json(&args.file);

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
	let endpoints = EndPoints::parse_json(&args.file);

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
