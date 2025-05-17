use std::io::Write;

const FISH_COMPLETE_TEMPLATE: &str = r#"
complete -e http
function __httpie_openapi_complete
		echo begin complete >~/httpie-complete.log
    set -l cmdline (commandline -cp)
    set -l cursor (commandline -C)
		echo "command: " "httpie-oapi complete  --file $OPENAPI_FILE --line "$cmdline" --cursor-pos $cursor" >~/httpie-complete.log
    httpie-oapi complete --file $OPENAPI_FILE --line "$cmdline" --cursor-pos $cursor
end

# Set OpenAPI file path
set -g OPENAPI_FILE ~/openapi.json

# Add OpenAPI-aware completion for http command
complete -c http -n 'not __fish_seen_argument -w GET -w POST -w PUT -w DELETE -w PATCH -w HEAD -w OPTIONS' \
    -a '(__httpie_openapi_complete)'

"#;

pub fn generate_completion(output: Option<String>) -> std::io::Result<()> {
	let mut writer: Box<dyn Write> = if let Some(path) = output {
		Box::new(std::fs::File::create(path)?)
	} else {
		Box::new(std::io::stdout())
	};

	writer.write_all(FISH_COMPLETE_TEMPLATE.as_bytes())?;
	writer.flush()?;
	Ok(())
}
