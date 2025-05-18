use std::path::Path;

use anyhow::{Context, Result};
use openapiv3::{OpenAPI, ReferenceOr, Schema};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::reference::resolve_schema_reference;
use super::{Method, Param};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndPoints(Vec<EndPoint>);

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

	pub fn fzf_list_format(&self, base_url: impl AsRef<str>) -> String {
		format!("{} {}{}", self.method, base_url.as_ref(), self.path)
	}

	pub fn fish_complete_format(&self, base_url: impl AsRef<str>) -> String {
		let summary = self.summary.as_deref().unwrap_or(&self.path);
		format!("{}{}\t{}", base_url.as_ref(), self.path, summary)
	}
}

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

		info!("Starting to parse OpenAPI endpoints");
		for (path_str, path_item) in &api.paths.paths {
			debug!("Processing path: {}", path_str);
			let path = match path_item {
				Item(p) => p,
				Reference { .. } => {
					debug!("Skipping referenced path: {}", path_str);
					continue;
				}
			};

			let common_params = Self::extract_parameters(&path.parameters, &api);
			debug!("Found {} common parameters for path: {}", common_params.len(), path_str);

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
				if op_opt.is_none() {
					continue;
				}
				let op = op_opt.as_ref().unwrap();
				debug!("Processing {} {} operation", method_ty, path_str);

				let mut params = common_params.clone();

				// Add operation-level parameters
				let op_params = Self::extract_parameters(&op.parameters, &api);
				debug!("Found {} operation parameters", op_params.len());
				params.extend(op_params);

				// Add request body parameters
				if let Some(request_body) = &op.request_body {
					let body_params = Self::extract_request_body_parameters(request_body, &api);
					debug!("Found {} request body parameters", body_params.len());
					params.extend(body_params);
				}

				endpoints.push(EndPoint {
					method: method_ty,
					path: path_str.clone(),
					summary: op.summary.clone(),
					params,
				});
			}
		}

		info!("Successfully parsed {} endpoints", endpoints.len());
		EndPoints(endpoints)
	}
}

impl EndPoints {
	fn extract_parameters(
		parameters: &[ReferenceOr<openapiv3::Parameter>],
		spec: &OpenAPI,
	) -> Vec<Param> {
		parameters.iter().filter_map(|p| Self::extract_parameter(p, spec)).collect()
	}

	fn extract_parameter(
		parameter: &ReferenceOr<openapiv3::Parameter>,
		spec: &OpenAPI,
	) -> Option<Param> {
		match parameter {
			ReferenceOr::Item(param) => Param::try_from(param).ok(),
			ReferenceOr::Reference { reference } => {
				debug!("Extracting referenced parameter: {}", reference);
				Self::extract_referenced_parameter(reference, spec)
			}
		}
	}

	fn extract_referenced_parameter(reference: &str, spec: &OpenAPI) -> Option<Param> {
		let schema = resolve_schema_reference(reference, spec).ok()?;
		let params = Param::try_from_schema(schema).ok()?;
		params.into_iter().next()
	}

	fn extract_schema_parameters(schema: &ReferenceOr<Schema>, spec: &OpenAPI) -> Vec<Param> {
		match schema {
			ReferenceOr::Item(schema) => {
				debug!("Processing direct schema");
				Param::try_from_schema(schema).unwrap_or_default()
			}
			ReferenceOr::Reference { reference } => {
				debug!("Resolving schema reference: {}", reference);
				match resolve_schema_reference(reference, spec) {
					Ok(resolved_schema) => {
						debug!("Successfully resolved schema reference");
						Param::try_from_schema(resolved_schema).unwrap_or_default()
					}
					Err(e) => {
						warn!("Failed to resolve schema reference: {}", e);
						Vec::new()
					}
				}
			}
		}
	}

	fn extract_request_body_parameters(
		request_body: &ReferenceOr<openapiv3::RequestBody>,
		spec: &OpenAPI,
	) -> Vec<Param> {
		match request_body {
			ReferenceOr::Item(body) => {
				if let Some(media_type) = body.content.get("application/json") {
					if let Some(schema) = &media_type.schema {
						debug!("Found request body schema");
						return Self::extract_schema_parameters(schema, spec);
					}
				}
				debug!("No request body schema found");
				Vec::new()
			}
			ReferenceOr::Reference { .. } => {
				warn!("Request body is a reference, which is not supported");
				Vec::new()
			}
		}
	}
}
