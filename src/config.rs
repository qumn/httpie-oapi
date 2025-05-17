use crate::openapi::ApiSpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub static CONFIG_DIR: &str = ".config/httpie-oapi";
pub static CACHE_DIR: &str = ".cache/httpie-oapi";
pub static CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	/// Map of service name to API specification
	apis: HashMap<String, ApiSpec>,
}

impl Config {
	pub fn load() -> Self {
		let config_path = Self::config_file();
		if !config_path.exists() {
			return Self { apis: HashMap::new() };
		}

		let content = fs::read_to_string(&config_path).unwrap_or_else(|e| {
			eprintln!("Failed to read config file: {}", e);
			std::process::exit(1);
		});

		toml::from_str(&content).unwrap_or_else(|e| {
			eprintln!("Failed to parse config file: {}", e);
			std::process::exit(1);
		})
	}

	pub fn config_file() -> PathBuf {
		let path = Self::config_dir().join(CONFIG_FILE);
		Self::ensure_dir_exists(&path);
		path
	}

	pub fn get_cache_path(name: &str) -> PathBuf {
		let path = Self::cache_dir().join(format!("{}.json", name));
		Self::ensure_dir_exists(&path);
		path
	}

	pub fn get_endpoints_cache_path(name: &str) -> PathBuf {
		let path = Self::cache_dir().join(format!("{}.endpoints.json", name));
		Self::ensure_dir_exists(&path);
		path
	}

	fn ensure_dir_exists(path: &Path) {
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent).unwrap_or_else(|e| {
				eprintln!("Failed to create directory: {}", e);
				std::process::exit(1);
			});
		}
	}

	fn config_dir() -> PathBuf {
		dirs::home_dir()
			.unwrap_or_else(|| {
				eprintln!("Could not determine home directory");
				std::process::exit(1);
			})
			.join(CONFIG_DIR)
	}

	fn cache_dir() -> PathBuf {
		dirs::home_dir()
			.unwrap_or_else(|| {
				eprintln!("Could not determine home directory");
				std::process::exit(1);
			})
			.join(CACHE_DIR)
	}
}

impl Config {
	pub fn save(&self) {
		let config_path = Self::config_file();
		// Ensure config directory exists
		if let Some(parent) = config_path.parent() {
			fs::create_dir_all(parent).unwrap_or_else(|e| {
				eprintln!("Failed to create config directory: {}", e);
				std::process::exit(1);
			});
		}

		let content = toml::to_string_pretty(self).unwrap_or_else(|e| {
			eprintln!("Failed to serialize config: {}", e);
			std::process::exit(1);
		});

		fs::write(&config_path, content).unwrap_or_else(|e| {
			eprintln!("Failed to write config file: {}", e);
			std::process::exit(1);
		});
	}

	pub fn add_api(&mut self, name: String, url: String, base_url: String) {
		let api_spec = ApiSpec::new(name.clone(), url, base_url);
		// cache the api
		api_spec.refresh_endpoints_cache();
		self.apis.insert(name, api_spec);
	}

	pub fn remove_api(&mut self, name: &str) -> bool {
		match self.apis.remove(name) {
			Some(_) => {
				// clear cache
				let cache_path = Self::get_cache_path(name);
				if cache_path.exists() {
					let _ = fs::remove_file(cache_path);
				}
				let endpoints_cache_path = Self::get_endpoints_cache_path(name);
				if endpoints_cache_path.exists() {
					let _ = fs::remove_file(endpoints_cache_path);
				}
				true
			}
			None => false,
		}
	}

	pub fn list_apis(&self) -> Vec<&ApiSpec> {
		self.apis.values().collect()
	}

	pub fn get_api(&self, name: &str) -> Option<&ApiSpec> {
		self.apis.get(name)
	}
}
