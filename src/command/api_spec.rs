use anyhow::anyhow;
use clap::{ArgAction, Args, Subcommand};

use crate::{config::Config, openapi::ApiSpec};

#[derive(Subcommand, Debug)]
pub(super) enum ApiSpecCommands {
	/// Add or update an OpenApi server
	Add(SaveArgs),
	/// Remove an OpenApi server
	#[command(visible_alias = "rm")]
	Remove(RemoveArgs),
	/// List all registered OpenApi server
	#[command(visible_alias = "ls")]
	List(ListArgs),
	/// Refresh OpenAPI cache for OpenApi server
	#[command(visible_alias = "sync")]
	Refresh(RefreshArgs),
}

#[derive(Args, Debug)]
pub(super) struct SaveArgs {
	/// Name of the API service to add or update
	/// This name will be used to identify the API in other commands
	#[arg(value_name = "NAME")]
	name: String,

	/// URL of the OpenAPI/Swagger specification
	/// This should be a valid URL pointing to a JSON or YAML OpenAPI document
	#[arg(value_name = "SPEC_URL")]
	spec_url: String,

	/// Base URL for the API service
	/// This is the root URL where the API endpoints will be accessed
	/// Example: https://api.example.com/v1
	#[arg(long, short, value_name = "BASE_URL")]
	base_url: String,

	/// Force update if the API already exists
	/// Without this flag, adding an existing API will fail
	#[arg(long, short = 'f', action = ArgAction::SetTrue)]
	force: bool,
}

#[derive(Args, Debug)]
pub(super) struct RefreshArgs {
	/// Names of the APIs to refresh cache
	/// If not provided, refreshes all registered APIs
	/// Example: httpie-oapi spec sync api1 api2
	#[arg(value_name = "NAMES")]
	names: Vec<String>,
}

#[derive(Args, Debug)]
pub(super) struct RemoveArgs {
	/// Name of the API service to remove
	/// This will remove both the API configuration and its cache
	#[arg(value_name = "NAME")]
	name: String,
}

#[derive(Args, Debug)]
pub(super) struct ListArgs {
	/// Show detailed information about each API
	/// Including spec URL, base URL, and cache file location
	#[arg(short, long)]
	detailed: bool,
}

impl ApiSpecCommands {
	pub(super) fn run(&self, config: &mut Config) -> anyhow::Result<()> {
		match self {
			ApiSpecCommands::Add(args) => Self::save_server(args, config),
			ApiSpecCommands::Remove(args) => Self::remove_server(args, config),
			ApiSpecCommands::List(args) => Self::list_server(args, config),
			ApiSpecCommands::Refresh(args) => Self::refresh_server(args, config),
		}
	}

	fn save_server(args: &SaveArgs, config: &mut Config) -> anyhow::Result<()> {
		// Check if API already exists
		if !args.force && config.get_api(&args.name).is_some() {
			return Err(anyhow!("Error: API '{}' already exists. Use --force to overwrite.", args.name));
		}

		let server = ApiSpec::new(args.name.clone(), args.spec_url.clone(), args.base_url.clone());

		// Force download and cache endpoints
		server.refresh_endpoints_cache();

		config.add_api(args.name.clone(), args.spec_url.clone(), args.base_url.clone());
		config.save();

		if args.force {
			println!("Updated API '{}' successfully", args.name);
		} else {
			println!("Added API '{}' successfully", args.name);
		}

		Ok(())
	}

	fn remove_server(args: &RemoveArgs, config: &mut Config) -> anyhow::Result<()> {
		if config.remove_api(&args.name) {
			config.save();
			println!("Removed API '{}' successfully", args.name);
			Ok(())
		} else {
			Err(anyhow!("API '{}' not found", args.name))
		}
	}

	fn list_server(args: &ListArgs, config: &Config) -> anyhow::Result<()> {
		let apis = config.list_apis();

		if apis.is_empty() {
			println!("No APIs registered");
			return Ok(());
		}

		for api in apis {
			if args.detailed {
				println!("Name: {}", api.name);
				println!("SPEC URL: {}", api.spec_url);
				println!("Base URL: {}", api.base_url);
				println!("Cache: {}", Config::get_cache_path(&api.name).display());
				println!();
			} else {
				println!("{}\t{}", api.name, api.spec_url);
			}
		}
		Ok(())
	}

	fn refresh_server(args: &RefreshArgs, config: &Config) -> anyhow::Result<()> {
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
		Ok(())
	}
}

