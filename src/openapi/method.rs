#[derive(Debug)]
pub enum Method {
	Get,
	Post,
	Put,
	Delete,
	Head,
	Patch,
	Options,
}

impl From<&str> for Method {
	fn from(s: &str) -> Self {
		match s.to_uppercase().as_str() {
			"GET" => Method::Get,
			"POST" => Method::Post,
			"PUT" => Method::Put,
			"DELETE" => Method::Delete,
			"PATCH" => Method::Patch,
			"HEAD" => Method::Head,
			"OPTIONS" => Method::Options,
			_ => panic!("Unsupported HTTP method: {}", s),
		}
	}
}

impl std::fmt::Display for Method {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let name = match self {
			Method::Get => "GET",
			Method::Post => "POST",
			Method::Put => "PUT",
			Method::Delete => "DELETE",
			Method::Head => "HEAD",
			Method::Patch => "PATCH",
			Method::Options => "OPTIONS",
		};
		write!(f, "{}", name)
	}
}
