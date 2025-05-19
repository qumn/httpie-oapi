use std::process::ExitCode;

use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod command;
mod config;
mod openapi;
mod tokens;

pub use command::Command;
pub use config::Config;

fn main() -> ExitCode {
	init_logging();
	let mut config = Config::load();
	let command = Command::parse();
	match command.run(&mut config) {
		Ok(_) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("{e:#}");
			ExitCode::FAILURE
		}
	}
}

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

	tracing::info!("Logging system initialized");
}
