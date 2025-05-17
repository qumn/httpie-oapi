use clap::Parser;
use crate::cli::{Cli, Commands, SpecCommand};
use crate::handlers::*;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod config;
mod fish;
mod handlers;
mod openapi;
mod command_tokens;

fn init_logging() {
	let log_dir = dirs::home_dir()
		.unwrap_or_else(|| {
			eprintln!("Could not determine home directory");
			std::process::exit(1);
		})
		.join(".local/state/httpie-oapi");

	std::fs::create_dir_all(&log_dir).unwrap_or_else(|e| {
		eprintln!("Failed to create log directory: {}", e);
		std::process::exit(1);
	});

	let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
		.rotation(tracing_appender::rolling::Rotation::DAILY)
		.filename_prefix("httpie-oapi")
		.filename_suffix("log")
		.build(log_dir)
		.unwrap_or_else(|e| {
			eprintln!("Failed to create log file: {}", e);
			std::process::exit(1);
		});

	tracing_subscriber::registry()
		.with(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
		.with(fmt::Layer::default().with_writer(file_appender))
		.init();

	tracing::info!("日志系统初始化完成");
}

fn main() {
	init_logging();
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
