use super::EndPoints;
use crate::config::Config;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSpec {
	/// Name of the API service
	pub name: String,
	/// URL of the OpenAPI/Swagger specification
	pub url: String,
	/// Base URL for the API service
	pub base_url: String,
	/// Cached endpoints, loaded on demand
	#[serde(skip)]
	endpoints: Option<EndPoints>,
}

impl ApiSpec {
	/// Create a new ApiSpec instance
	pub fn new(name: String, url: String, base_url: String) -> Self {
		Self { name, url, base_url, endpoints: None }
	}

	/// Get the endpoints for this API spec. If cached in memory, return that.
	/// Otherwise try to load from file cache, and if that fails, download and parse.
	pub fn get_endpoints(&mut self) -> &EndPoints {
		if self.endpoints.is_none() {
			let endpoints_cache_path = Config::get_endpoints_cache_path(&self.name);

			// Try to load from file cache first
			if endpoints_cache_path.exists() {
				if let Ok(endpoints) = EndPoints::try_from_json(&endpoints_cache_path) {
					self.endpoints = Some(endpoints);
				}
			}

			// If still none, download and parse OpenAPI spec
			if self.endpoints.is_none() {
				self.endpoints = Some(self.refresh_endpoints_cache());
			}
		}

		self.endpoints.as_ref().unwrap()
	}

	/// Force download the OpenAPI spec and update both file and memory cache
	pub fn refresh_endpoints_cache(&self) -> EndPoints {
		// Validate URL
		let url = Url::parse(&self.url).unwrap_or_else(|e| {
			eprintln!("Invalid OpenAPI URL '{}': {}", self.url, e);
			std::process::exit(1);
		});

		// Download OpenAPI spec
		let client = Client::new();
		let response = client.get(url).send().unwrap_or_else(|e| {
			eprintln!("Failed to fetch OpenAPI spec: {}", e);
			eprintln!("Please verify that the Swagger/OpenAPI URL '{}' is correct and accessible", self.url);
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
			eprintln!("Please verify that the URL '{}' points to a valid Swagger/OpenAPI specification", self.url);
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
