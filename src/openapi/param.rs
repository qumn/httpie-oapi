use openapiv3::Parameter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamSource {
	Query,
	Body,
	Path,
	Header,
	Form,
}

impl ParamSource {
	pub fn httpie_operator(&self) -> &'static str {
		match self {
			ParamSource::Body | ParamSource::Form | ParamSource::Path => "=",
			ParamSource::Query => "==",
			ParamSource::Header => ":",
		}
	}
}

impl From<&str> for ParamSource {
	fn from(s: &str) -> Self {
		match s.to_lowercase().as_str() {
			"query" => Self::Query,
			"body" => Self::Body,
			"path" => Self::Path,
			"header" => Self::Header,
			"form" => Self::Form,
			_ => panic!("Unsupported Param Source: {}", s),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
	pub name: String,
	pub required: bool,
	pub source: ParamSource,
	pub description: Option<String>,
}

impl Param {
	pub fn fish_complete_format(&self) -> String {
		let desc = self.description.as_deref().unwrap_or(&self.name);
		let desc = if self.required { desc.to_string() } else { format!("[{}]", desc) };

		format!("{}{}\t{}", self.name, self.source.httpie_operator(), desc)
	}
}

impl std::fmt::Display for Param {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.fish_complete_format())
	}
}

impl TryFrom<&Parameter> for Param {
	type Error = String;

	fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
		let (parameter_data, source) = match parameter {
			Parameter::Query { parameter_data, .. } => (parameter_data, ParamSource::Query),
			Parameter::Header { parameter_data, .. } => (parameter_data, ParamSource::Header),
			Parameter::Path { parameter_data, .. } => (parameter_data, ParamSource::Path),
			Parameter::Cookie { .. } => return Err("unsupported Cookie param".to_owned()),
		};
		Ok(Self {
			name: parameter_data.name.clone(),
			required: parameter_data.required,
			source,
			description: parameter_data.description.clone(),
		})
	}
}
