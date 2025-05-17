use clap::Parser;

mod cli;
mod fish;
mod handlers;
mod openapi;

use crate::cli::*;

fn main() {
	let cli = Cli::parse();

	match &cli.command {
		Commands::Path(args) => handlers::handle_path_command(args),
		Commands::Param(args) => handlers::handle_param_command(args),
		Commands::Complete(args) => handlers::handle_complete(args),
		Commands::Completions(args) => handlers::handle_completions(args),
	}
}
