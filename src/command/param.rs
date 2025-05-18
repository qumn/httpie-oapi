use anyhow::Context;
use clap::{ArgAction, Args};

use crate::config::Config;

#[derive(Args, Debug)]
pub struct ParamCommand {
	/// Name of the API service
	#[arg(short, long, value_name = "NAME")]
	name: String,

	/// The API path to extract parameters from (e.g. `/users/{id}`)
	#[arg(long, value_name = "PATH")]
	path: String,

	/// Optional pattern to filter parameters
	#[arg(long, value_name = "PATTERN")]
	pattern: Option<String>,

	/// Output in fish shell completion format
	#[arg(long, action = ArgAction::SetTrue, conflicts_with = "fzf")]
	fish: bool,

	/// Output in fzf-friendly list format (default)
	#[arg(long, action = ArgAction::SetTrue, conflicts_with = "fish")]
	fzf: bool,
}

impl ParamCommand {
	pub(super) fn run(&self, config: &Config) -> anyhow::Result<()> {
		let api =
			config.get_api(&self.name).with_context(|| format!("API '{}' not found", self.name))?;
		let endpoints = api.get_endpoints();
		let ep = endpoints
			.find(&self.path)
			.with_context(|| format!("No endpoint matched path '{}'", self.path))?;

		let mut filtered_params: Vec<_> = if let Some(pat) = &self.pattern {
			ep.params.iter().filter(|param| param.name.contains(pat)).collect()
		} else {
			ep.params.iter().collect()
		};

		filtered_params.sort_by_key(|&p| !p.required);

		for param in filtered_params {
			if self.fish {
				println!("{}", param);
			} else {
				println!("{}", param.fish_complete_format());
			}
		}
		Ok(())
	}
}
