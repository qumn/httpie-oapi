use anyhow::{Result, anyhow};
use openapiv3::{OpenAPI, ReferenceOr, Schema};
use tracing::{debug, warn};

/// Resolve schema reference
pub fn resolve_schema_reference<'a>(reference: &str, spec: &'a OpenAPI) -> Result<&'a Schema> {
	debug!("Attempting to resolve schema reference: {}", reference);

	if !reference.starts_with("#/components/schemas/") {
		warn!("Invalid schema reference path: {}", reference);
		return Err(anyhow!("Not a schema reference: {}", reference));
	}

	let schema_name = reference.trim_start_matches("#/components/schemas/");
	debug!("Looking for schema: {}", schema_name);

	let schema = spec
		.components
		.as_ref()
		.and_then(|components| components.schemas.get(schema_name))
		.and_then(|schema_ref| match schema_ref {
			ReferenceOr::Item(schema) => {
				debug!("Found schema: {}", schema_name);
				Some(schema)
			}
			ReferenceOr::Reference { .. } => {
				warn!("Schema {} is a reference to another reference, which is not supported", schema_name);
				None
			}
		})
		.ok_or_else(|| {
			warn!("Schema not found: {}", schema_name);
			anyhow!("Schema not found: {}", schema_name)
		})?;

	Ok(schema)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_resolve_schema_reference() {
		let spec = json!({
				"openapi": "3.0.0",
				"info": {
						"title": "Test API",
						"version": "1.0.0"
				},
				"paths": {},
				"components": {
						"schemas": {
								"User": {
										"type": "object",
										"properties": {
												"name": { "type": "string" }
										}
								}
						}
				}
		});

		let spec = serde_json::from_value::<OpenAPI>(spec).unwrap();
		let reference = "#/components/schemas/User";
		let resolved = resolve_schema_reference(reference, &spec).unwrap();
		match resolved.schema_kind {
			openapiv3::SchemaKind::Type(openapiv3::Type::Object(_)) => {}
			_ => unreachable!("Expected object type"),
		}
	}

	#[test]
	fn test_resolve_schema_reference_not_found() {
		let spec = json!({
				"openapi": "3.0.0",
				"info": {
						"title": "Test API",
						"version": "1.0.0"
				},
				"paths": {},
				"components": {
						"schemas": {
								"User": {
										"type": "object",
										"properties": {
												"name": { "type": "string" }
										}
								}
						}
				}
		});

		let spec = serde_json::from_value::<OpenAPI>(spec).unwrap();
		let reference = "#/components/schemas/NonExistent";
		let result = resolve_schema_reference(reference, &spec);
		assert!(result.is_err());
	}

	#[test]
	fn test_resolve_schema_reference_invalid_path() {
		let spec = json!({
				"openapi": "3.0.0",
				"info": {
						"title": "Test API",
						"version": "1.0.0"
				},
				"paths": {},
				"components": {
						"schemas": {
								"User": {
										"type": "object",
										"properties": {
												"name": { "type": "string" }
										}
								}
						}
				}
		});

		let spec = serde_json::from_value::<OpenAPI>(spec).unwrap();
		let reference = "#/invalid/path/User";
		let result = resolve_schema_reference(reference, &spec);
		assert!(result.is_err());
	}
}

