use openapiv3::{Parameter, Schema, SchemaKind, Type};
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
	pub fn httpie_param_prefix(&self) -> &'static str {
		match self {
			ParamSource::Path => ":",
			_ => "",
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

		format!("{}\t{}", self.httpie_param_format(), desc)
	}

	pub fn httpie_param_format(&self) -> String {
		format!("{}{}{}", self.source.httpie_param_prefix(), self.name, self.source.httpie_operator())
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

impl Param {
	pub fn try_from_schema(schema: &Schema) -> Result<Vec<Self>, String> {
		match &schema.schema_kind {
			SchemaKind::Type(Type::Object(object_type)) => {
				let mut params = Vec::new();
				for (name, property) in &object_type.properties {
					let required = object_type.required.contains(name);
					let description = match property {
						openapiv3::ReferenceOr::Item(schema) => schema.schema_data.description.clone(),
						openapiv3::ReferenceOr::Reference { .. } => None,
					};
					params.push(Self {
						name: name.clone(),
						required,
						source: ParamSource::Body,
						description,
					});
				}
				Ok(params)
			}
			_ => Err("Schema must be an object type".to_string()),
		}
	}
}
