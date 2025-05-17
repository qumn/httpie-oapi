use clap::Parser;
use crate::cli::{Cli, Commands, SpecCommand};
use crate::handlers::*;

mod cli;
mod config;
mod fish;
mod handlers;
mod openapi;

fn main() {
	let cli = Cli::parse();

	match cli.command {
		Commands::Path(args) => handle_path_command(&args),
		Commands::Param(args) => handle_param_command(&args),
		Commands::Complete(args) => handle_complete(&args),
		Commands::Completions(args) => handle_completions(&args),
		Commands::Spec(cmd) => match cmd {
			SpecCommand::Save(args) => handle_save_api(&args),
			SpecCommand::Remove(args) => handle_remove_api(&args),
			SpecCommand::List(args) => handle_list_apis(&args),
			SpecCommand::Refresh(args) => handle_refresh_api(&args),
		},
	}
}
