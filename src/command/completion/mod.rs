use anyhow::anyhow;
use clap::{Parser, ValueEnum};
mod fish;

#[derive(Parser, Debug)]
pub struct CompletionsCommand {
	/// Shell to generate completions for
	#[arg(value_enum)]
	pub shell: Shell,

	/// Output file path, default to stdout
	pub output: Option<String>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Shell {
	Fish,
	// TODO: Support for other shells can be added in the future
}

impl CompletionsCommand {
	pub(super) fn run(&self) -> anyhow::Result<()> {
		match self.shell {
			Shell::Fish => {
				if let Err(e) = fish::generate_completion(self.output.clone()) {
					return Err(anyhow!("Failed to generate fish completion: {}", e));
				}
			}
		}
		Ok(())
	}
}
