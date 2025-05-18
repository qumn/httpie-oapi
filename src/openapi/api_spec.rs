use crate::config::Config;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use url::Url;

use super::EndPoints;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSpec {
	/// Name of the API service
	pub name: String,
	/// URL of the OpenAPI/Swagger specification
	pub spec_url: String,
	/// Base URL for the API service
	pub base_url: String,
	/// Cached endpoints, loaded on demand
	#[serde(skip)]
	endpoints: RefCell<Option<EndPoints>>,
}

// 同步修改所有相关方法名
impl ApiSpec {
	// ← 同步修改
	/// Create a new ApiSpec instance
	pub fn new(name: String, spec_url: String, base_url: String) -> Self {
		// ← 参数名调整
		Self { name, spec_url, base_url, endpoints: RefCell::new(None) }
	}

	/// Get the endpoints for this API spec. If cached in memory, return that.
	/// Otherwise try to load from file cache, and if that fails, download and parse.
	pub fn get_endpoints(&self) -> EndPoints {
		if self.endpoints.borrow().is_none() {
			let endpoints_cache_path = Config::get_endpoints_cache_path(&self.name);

			// Try to load from file cache first
			if endpoints_cache_path.exists() {
				if let Ok(endpoints) = EndPoints::try_from_json(&endpoints_cache_path) {
					*self.endpoints.borrow_mut() = Some(endpoints);
				}
			}

			// If still none, download and parse OpenAPI spec
			if self.endpoints.borrow().is_none() {
				let endpoints = self.refresh_endpoints_cache();
				*self.endpoints.borrow_mut() = Some(endpoints);
			}
		}

		self.endpoints.borrow().as_ref().unwrap().clone()
	}

	/// Force download the OpenAPI spec and update both file and memory cache
	pub fn refresh_endpoints_cache(&self) -> EndPoints {
		// Validate URL
		let url = Url::parse(&self.spec_url).unwrap_or_else(|e| {
			eprintln!("Invalid OpenAPI URL '{}': {}", self.spec_url, e);
			std::process::exit(1);
		});

		// Download OpenAPI spec
		let client = Client::new();
		let response = client.get(url).send().unwrap_or_else(|e| {
			eprintln!("Failed to fetch OpenAPI spec: {}", e);
			eprintln!(
				"Please verify that the Swagger/OpenAPI URL '{}' is correct and accessible",
				self.spec_url
			);
			std::process::exit(1);
		});

		// Check response status
		if !response.status().is_success() {
			eprintln!(
				"Failed to fetch OpenAPI spec: HTTP {} - {}",
				response.status(),
				response.status().canonical_reason().unwrap_or("Unknown error")
			);
			std::process::exit(1);
		}

		let spec_json = response.text().unwrap_or_else(|e| {
			eprintln!("Failed to read OpenAPI spec: {}", e);
			std::process::exit(1);
		});

		// Parse OpenAPI spec
		let endpoints: EndPoints = EndPoints::try_from_openapi(&spec_json).unwrap_or_else(|e| {
			eprintln!("Failed to parse OpenAPI JSON: {}", e);
			eprintln!(
				"Please verify that the URL '{}' points to a valid Swagger/OpenAPI specification",
				self.spec_url
			);
			std::process::exit(1);
		});

		// Save OpenAPI spec to cache
		let cache_path = Config::get_cache_path(&self.name);
		std::fs::write(&cache_path, &spec_json).unwrap_or_else(|e| {
			eprintln!("Failed to write cache file: {}", e);
			std::process::exit(1);
		});

		let endpoints_cache_path = Config::get_endpoints_cache_path(&self.name);
		endpoints.save_to_file(&endpoints_cache_path).unwrap_or_else(|e| {
			eprintln!("Failed to write endpoints cache file: {}", e);
			std::process::exit(1);
		});

		endpoints
	}
}
