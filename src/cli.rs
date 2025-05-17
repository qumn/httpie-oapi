use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

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
	/// Manage OpenAPI specifications
	#[command(subcommand)]
	Spec(SpecCommand),
}

#[derive(Subcommand, Debug)]
pub enum SpecCommand {
	/// Save (add or update) an API specification
	Save(SaveApiArgs),
	/// Remove an API specification
	Remove(RemoveApiArgs),
	/// List all registered API specifications
	List(ListApiArgs),
	/// Refresh OpenAPI cache for specified APIs
	Refresh(RefreshApiArgs),
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Shell {
	Fish,
	// Support for other shells can be added in the future
}

#[derive(Args, Debug)]
pub struct PathArgs {
	/// Name of the API service
	#[arg(short, long, value_name = "NAME")]
	pub name: String,

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
	/// Name of the API service
	#[arg(short, long, value_name = "NAME")]
	pub name: String,

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
	/// Name of the API service
	#[arg(short, long)]
	pub name: String,

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

#[derive(Args, Debug)]
pub struct SaveApiArgs {
	/// Name of the API service
	pub name: String,
	/// URL of the OpenAPI/Swagger specification
	pub url: String,
	/// Base URL for the API service
	#[arg(long)]
	pub base_url: String,
	/// Force update if the API already exists
	#[arg(long, action = ArgAction::SetTrue)]
	pub force: bool,
}

#[derive(Args, Debug)]
pub struct RefreshApiArgs {
	/// Names of the APIs to refresh cache. If not provided, refreshes all APIs.
	pub names: Vec<String>,
}

#[derive(Args, Debug)]
pub struct RemoveApiArgs {
	/// Name of the API service to remove
	pub name: String,
}

#[derive(Args, Debug)]
pub struct ListApiArgs {
	/// Show detailed information
	#[arg(short, long)]
	pub detailed: bool,
}

