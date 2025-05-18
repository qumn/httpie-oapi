use anyhow::Context;
use clap::{ArgAction, Args};
use tracing::debug;

use crate::config::Config;
use crate::openapi::ApiSpec;

#[derive(Args, Debug)]
pub struct PathCommand {
	/// Name of the API service (optional, show all APIs if not provided)
	#[arg(short, long, value_name = "NAME")]
	name: Option<String>,

	/// Optional filter to match specific paths
	#[arg(long, value_name = "PATTERN")]
	pattern: Option<String>,

	/// Output in fish shell completion format
	#[arg(long, action = ArgAction::SetTrue, conflicts_with = "fzf")]
	fish: bool,

	/// Output in fzf-friendly list format (default)
	#[arg(long, action = ArgAction::SetTrue, conflicts_with = "fish")]
	fzf: bool,
}

impl PathCommand {
	pub(super) fn run(&self, config: &Config) -> anyhow::Result<()> {
		match &self.name {
			Some(name) => {
				// Show paths for a specific API
				debug!("Showing paths for API: {}", name);
				let api = config.get_api(name)
					.with_context(|| format!("API '{}' not found", name))?;
				self.show_api_paths(api)
			}
			None => {
				// Show paths for all APIs
				debug!("Showing paths for all APIs");
				for api in config.list_apis() {
					self.show_api_paths(api)?;
				}
				Ok(())
			}
		}
	}

	fn show_api_paths(&self, api: &ApiSpec) -> anyhow::Result<()> {
		let endpoints = api.get_endpoints();
		let filtered: Vec<_> = if let Some(pattern) = &self.pattern {
			endpoints.filter(pattern)
		} else {
			endpoints.all()
		};

		for endpoint in filtered {
			if self.fish {
				println!("{}", endpoint.fish_complete_format(&api.base_url));
			} else {
				println!("{}", endpoint.fzf_list_format(&api.base_url));
			}
		}

		Ok(())
	}
}
