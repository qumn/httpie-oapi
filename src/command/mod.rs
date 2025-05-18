mod api_spec;
mod complete;
mod completion;
mod param;
mod path;
mod path_var;

use api_spec::ApiSpecCommands;
use clap::Parser;
use complete::CompleteCommand;
use completion::CompletionsCommand;
use param::ParamCommand;
use path::PathCommand;
use path_var::PathVarCommand;

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(name = "httpie-oapi", author, version, about = "OpenAPI-aware completion for HTTPie")]
pub struct Command {
	#[command(subcommand)]
	sub_command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
	/// List all paths from OpenAPI spec
	Path(PathCommand),
	/// List all parameters for a specific path
	Param(ParamCommand),
	/// Internal command for shell completion
	Complete(CompleteCommand),
	/// Generate shell completion scripts
	Completions(CompletionsCommand),
	/// Manage OpenAPI specifications
	#[command(subcommand)]
	Spec(ApiSpecCommands),
	/// Process path variables in URLs
	#[command(name = "path-var")]
	PathVar(PathVarCommand),
}

impl Command {
	pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
		match &self.sub_command {
			Commands::Path(path_command) => path_command.run(config),
			Commands::Param(param_command) => param_command.run(config),
			Commands::Complete(complete_command) => complete_command.run(config),
			Commands::Completions(completions_command) => completions_command.run(),
			Commands::Spec(spec_command) => spec_command.run(config),
			Commands::PathVar(path_var_command) => path_var_command.run(),
		}
	}
}
