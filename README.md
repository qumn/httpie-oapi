# httpie-oapi

A command-line tool that enhances HTTPie with OpenAPI specification support, providing intelligent command-line completion and API documentation integration.

## Features

- 🔍 Smart command-line completion for OpenAPI endpoints
- 📚 Automatic API documentation integration
- 🚀 Seamless integration with HTTPie
- 🐟 Fish shell completion support
- 🔄 Automatic API specification caching

## TODO Features

- [ ] Support for multiple OpenAPI specification formats (YAML, JSON)
- [ ] Support for OpenAPI $ref references
- [x] Fish shell completion support
- [ ] Zsh shell completion support
- [ ] Bash shell completion support
- [ ] Interactive mode for API exploration
- [ ] Support for API authentication methods
- [ ] Request/response validation against OpenAPI schema
- [ ] Support for environment variables in API specifications
- [ ] API documentation generation in markdown format
- [ ] Support for API versioning
- [ ] Rate limiting and request throttling

## Installation

```bash
cargo install httpie-oapi
```

## Usage

### Basic Commands

```bash
# List all registered APIs
httpie-oapi spec list

# Add a new API specification
httpie-oapi spec save --name myapi --url https://api.example.com/openapi.json --base-url https://api.example.com

# Remove an API specification
httpie-oapi spec remove myapi

# Refresh API specification cache
httpie-oapi spec refresh myapi
```

### Command Line Completion

The tool provides intelligent command-line completion based on the OpenAPI specification:

1. When no base URL is matched, it shows all available API specifications
2. When the cursor is on a base URL token, it shows all paths for that API
3. When both base URL and path are matched, it shows all available parameters

Example:
```bash
# Shows all registered APIs
httpie-oapi complete "http "

# Shows all paths for the matched API
httpie-oapi complete "http https://api.example.com"

# Shows all parameters for the matched path
httpie-oapi complete "http https://api.example.com/users"
```

### Fish Shell Integration

Generate fish shell completion:

```bash
httpie-oapi completions --shell fish --output ~/.config/fish/completions/httpie-oapi.fish
```

## Configuration

The tool stores API specifications and configuration in:
- `~/.config/httpie-oapi/config.toml` - Configuration file
- `~/.local/state/httpie-oapi/` - API specification cache

## Development

```bash
# Clone the repository
git clone https://github.com/yourusername/httpie-oapi.git

# Build the project
cargo build

# Run tests
cargo test
```

## License

MIT License 