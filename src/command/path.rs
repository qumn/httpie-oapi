use anyhow::Context;
use clap::{ArgAction, Args};

use crate::config::Config;

#[derive(Args, Debug)]
pub struct PathCommand {
	/// Name of the API service
	#[arg(short, long, value_name = "NAME")]
	name: String,

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
		let api =
			config.get_api(&self.name).with_context(|| format!("API '{}' not found", self.name))?;

		let endpoints = api.get_endpoints();
		let filtered: Vec<_> =
			if let Some(pattern) = &self.pattern { endpoints.filter(pattern) } else { endpoints.all() };

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
