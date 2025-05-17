use super::Method;
use super::Param;
use anyhow::{Context, Result};
use openapiv3::{OpenAPI, ReferenceOr};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndPoint {
	pub method: Method,
	pub path: String,
	pub summary: Option<String>,
	pub params: Vec<Param>,
}

impl EndPoint {
	pub fn get_params_sort(&self) -> Vec<Param> {
		let mut sorted = self.params.clone();
		sorted.sort_by_key(|param| !param.required);
		sorted
	}

	pub fn fzf_list_format(&self) -> String {
		format!("{} {}", self.method, self.path)
	}

	pub fn fish_complete_format(&self) -> String {
		let summary = self.summary.as_deref().unwrap_or(&self.path);
		format!("{}\t{}", self.path, summary)
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndPoints(Vec<EndPoint>);

impl EndPoints {
	pub fn filter(&self, path: impl AsRef<str>) -> Vec<&EndPoint> {
		self.0.iter().filter(|&endpoint| endpoint.path.contains(path.as_ref())).collect()
	}

	pub fn find(&self, path: impl AsRef<str>) -> Option<&EndPoint> {
		self.0.iter().find(|e| e.path == path.as_ref())
	}

	pub fn all(&self) -> Vec<&EndPoint> {
		self.0.iter().collect()
	}

	pub fn try_from_openapi(data: impl AsRef<str>) -> Result<Self> {
		let openapi: OpenAPI = serde_json::from_str(data.as_ref())?;
		Ok(EndPoints::from(openapi))
	}

	/// Try to parse endpoints from a JSON file, returning Result
	pub fn try_from_json(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		let data = std::fs::read_to_string(path)
			.with_context(|| format!("Failed to read endpoints file: {}", path.display()))?;

		serde_json::from_str(&data)
			.with_context(|| format!("Failed to parse endpoints JSON from file: {}", path.display()))
	}

	pub fn save_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
		let content = serde_json::to_string_pretty(self)?;
		std::fs::write(path, content)
	}
}

impl From<OpenAPI> for EndPoints {
	fn from(api: OpenAPI) -> Self {
		use ReferenceOr::*;
		let mut endpoints = vec![];

		for (path_str, path_item) in &api.paths.paths {
			let path = match path_item {
				Item(p) => p,
				Reference { .. } => continue,
			};

			let common_params: Vec<Param> = path
				.parameters
				.iter()
				.filter_map(|p| match p {
					Item(param) => Some(Param::try_from(param)),
					_ => None,
				})
				.flatten()
				.collect();

			let methods = vec![
				(Method::Get, &path.get),
				(Method::Post, &path.post),
				(Method::Put, &path.put),
				(Method::Delete, &path.delete),
				(Method::Patch, &path.patch),
				(Method::Head, &path.head),
				(Method::Options, &path.options),
			];

			for (method_ty, op_opt) in methods {
				if let Some(op) = op_opt {
					let mut params = common_params.clone();

					for p in &op.parameters {
						if let Item(param) = p {
							let Ok(param) = Param::try_from(param) else { continue };
							params.push(param);
						}
					}

					endpoints.push(EndPoint {
						method: method_ty,
						path: path_str.clone(),
						summary: op.summary.clone(),
						params,
					});
				}
			}
		}

		EndPoints(endpoints)
	}
}
