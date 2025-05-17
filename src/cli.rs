use clap::{ArgAction, Args, Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "httpie-oapi", author, version, about = "OpenAPI-aware completion for HTTPie")]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
	/// List all paths from OpenAPI spec
	Path(PathArgs),
	/// List all parameters for a specific path
	Param(ParamArgs),
	/// Internal command for shell completion
	Complete(CompleteArgs),
	/// Generate shell completion scripts
	Completions(CompletionsArgs),
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Shell {
	Fish,
	// Support for other shells can be added in the future
}

#[derive(Args, Debug)]
pub struct PathArgs {
	/// Path to the OpenAPI JSON file
	#[arg(short, long, value_name = "FILE")]
	pub file: String,

	/// Optional filter to match specific paths
	#[arg(long, value_name = "PATTERN")]
	pub pattern: Option<String>,

	/// Output in fish shell completion format
	#[arg(long, action = ArgAction::SetTrue, conflicts_with = "fzf")]
	pub fish: bool,

	/// Output in fzf-friendly list format (default)
	#[arg(long, action = ArgAction::SetTrue, conflicts_with = "fish")]
	pub fzf: bool,
}

#[derive(Args, Debug)]
pub struct ParamArgs {
	/// Path to the OpenAPI JSON file
	#[arg(short, long, value_name = "FILE")]
	pub file: String,

	/// The API path to extract parameters from (e.g. `/users/{id}`)
	#[arg(long, value_name = "PATH")]
	pub path: String,

	/// Optional pattern to filter parameters
	#[arg(long, value_name = "PATTERN")]
	pub pattern: Option<String>,

	/// Output in fish shell completion format
	#[arg(long, action = ArgAction::SetTrue, conflicts_with = "fzf")]
	pub fish: bool,

	/// Output in fzf-friendly list format (default)
	#[arg(long, action = ArgAction::SetTrue, conflicts_with = "fish")]
	pub fzf: bool,
}

#[derive(Args, Debug)]
pub struct CompleteArgs {
	/// OpenAPI JSON file path
	#[arg(short, long)]
	pub file: String,

	/// The current command line input
	#[arg(long, value_name = "LINE")]
	pub line: String,

	/// The current cursor position in the command line
	#[arg(long)]
	pub cursor_pos: usize,
}

#[derive(Parser, Debug)]
pub struct CompletionsArgs {
	/// Shell to generate completions for
	#[arg(value_enum)]
	pub shell: Shell,

	/// Output file path, default to stdout
	pub output: Option<String>,
}
