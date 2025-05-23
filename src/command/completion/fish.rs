use std::io::Write;

const FISH_COMPLETE_TEMPLATE: &str = r#"
# Override http command to handle path variables
function http --wraps http
    set -l arguments (httpie-oapi path-var -- $argv)
    eval command http $arguments
end

# Function to select an endpoint using fzf and convert it to http command
function h
    # Get all endpoints and pipe to fzf
    set -l selected (httpie-oapi path | fzf --height 60% --border --preview 'echo {}' --preview-window=down:3:wrap)
    
    if test -n "$selected"
        # Split the selected line into method and url
        set -l parts (string split ' ' -- $selected)
        if test (count $parts) -ge 2
            set -l method $parts[1]
            set -l url $parts[2..-1]
            
            # Get default options from environment variable
            set -l default_opts (set -q HTTPIE_DEFAULT_OPTS && echo $HTTPIE_DEFAULT_OPTS || echo "")
            
            # Convert to http command with default options and insert into command line
            commandline -r "http $default_opts $method $url "
        end
    end
end

function __httpie_openapi_complete
    set -l cmdline (commandline -cp)
    set -l cursor (commandline -C)
    httpie-oapi complete --line "$cmdline" --cursor-pos $cursor
end

# Function to get file completions with prefix
function __httpie_file_complete
    set -l current_token (commandline -t)
    # Split the token at @ and get the prefix and path
    set -l parts (string split '@' -- $current_token)

    if test (count $parts) -gt 1
        set -l prefix $parts[1]
        set -l path $parts[2]

        # if path is empty, use current directory
        if test -z "$path"
            set path "$PWD"
        end

        # Expand ~ to home directory and normalize path
        set path (path normalize (string replace -r '^~' $HOME -- $path))

        # Get the directory part of the path
        set -l dir
        if test -d "$path"
            set dir "$path"
        else
            set dir (path dirname $path)
        end

        # List files and directories with max depth of 2
        set -l items (ls -1 $dir 2>/dev/null)

        # Define path prefix based on dir
        set -l path_prefix
        if test "$dir" = "$PWD"
            set path_prefix ""
        else
            # Convert absolute path back to ~ if it's in home directory
            set path_prefix "$(string replace -r "^$HOME" '~' -- (path normalize "$dir"))/"
        end

        for item in $items
			set -l suffix "" 
			if test -d "$dir/$item"
				set suffix "/"
			end
            echo $prefix@$path_prefix$item$suffix
        end
    end
end

# Function to check if we need file completion
function __httpie_need_file_completion
    set -l current_token (commandline -ct)
    # Enable file completion if current token contains @
    string match -q -r '@' -- $current_token
end

# the content come from https://github.com/httpie/cli/blob/master/extras/httpie-completion.fish
function __fish_httpie_styles
    printf '%s\n' abap algol algol_nu arduino auto autumn borland bw colorful default emacs friendly fruity gruvbox-dark gruvbox-light igor inkpot lovelace manni material monokai murphy native paraiso-dark paraiso-light pastie perldoc pie pie-dark pie-light rainbow_dash rrt sas solarized solarized-dark solarized-light stata stata-dark stata-light tango trac vim vs xcode zenburn
end

function __fish_httpie_mime_types
    test -r /usr/share/mime/types && cat /usr/share/mime/types
end

function __fish_httpie_print_args
    set -l arg (commandline -t)
    string match -qe H "$arg" || echo -e $arg"H\trequest headers"
    string match -qe B "$arg" || echo -e $arg"B\trequest body"
    string match -qe h "$arg" || echo -e $arg"h\tresponse headers"
    string match -qe b "$arg" || echo -e $arg"b\tresponse body"
    string match -qe m "$arg" || echo -e $arg"m\tresponse metadata"
end

function __fish_httpie_auth_types
    echo -e "basic\tBasic HTTP auth"
    echo -e "digest\tDigest HTTP auth"
    echo -e "bearer\tBearer HTTP Auth"
end

function __fish_http_verify_options
    echo -e "yes\tEnable cert verification"
    echo -e "no\tDisable cert verification"
end

# Why we don't use complete -w:
# 1. When using complete -w, commandline -cp returns the wrapped command (http)
#    while commandline -C returns cursor position based on original command (https)
# 2. This mismatch causes incorrect cursor position calculation in __httpie_openapi_complete
# 3. For example, if user types "https api.example.com/users" with cursor at "users",
#    commandline -cp returns "http api.example.com/users" but commandline -C returns
#    position based on "https api.example.com/users"
# 4. This leads to incorrect completion results because cursor position doesn't match
#    the command being processed
# 5. Therefore, we need to set up complete rules separately for http and https to ensure
#    consistent command and cursor position handling
for cmd in http https
    # Remove default http completion
    complete -e $cmd
    # Add OpenAPI-aware completion for http command
    # -f: disable file completion by default
    complete -c $cmd -f -n 'not __fish_seen_argument -w GET -w POST -w PUT -w DELETE -w PATCH -w HEAD -w OPTIONS; and not __httpie_need_file_completion' \
        -a '(__httpie_openapi_complete)'

    # Add custom file completion that preserves the prefix
    complete -c $cmd -f -n __httpie_need_file_completion -a '(__httpie_file_complete)'

    complete -c $cmd -s j -l json -d 'Data items are serialized as a JSON object'
    complete -c $cmd -s f -l form -d 'Data items are serialized as form fields'
    complete -c $cmd -l multipart -d 'Always sends a multipart/form-data request'
    complete -c $cmd -l boundary -x -d 'Custom boundary string for multipart/form-data requests'
    complete -c $cmd -l raw -x -d 'Pass raw request data without extra processing'


    # Content Processing Options

    complete -c $cmd -s x -l compress -d 'Content compressed with Deflate algorithm'


    # Output Processing

    complete -c $cmd -l pretty -xa "all colors format none" -d 'Controls output processing'
    complete -c $cmd -s s -l style -xa "(__fish_httpie_styles)" -d 'Output coloring style'
    complete -c $cmd -l unsorted -d 'Disables all sorting while formatting output'
    complete -c $cmd -l sorted -d 'Re-enables all sorting options while formatting output'
    complete -c $cmd -l response-charset -x -d 'Override the response encoding'
    complete -c $cmd -l response-mime -xa "(__fish_httpie_mime_types)" -d 'Override the response mime type for coloring and formatting'
    complete -c $cmd -l format-options -x -d 'Controls output formatting'


    # Output Options

    complete -c $cmd -s p -l print -xa "(__fish_httpie_print_args)" -d 'String specifying what the output should contain'
    complete -c $cmd -s h -l headers -d 'Print only the response headers'
    complete -c $cmd -s m -l meta -d 'Print only the response metadata'
    complete -c $cmd -s b -l body -d 'Print only the response body'
    complete -c $cmd -s v -l verbose -d 'Print the whole request as well as the response'
    complete -c $cmd -l all -d 'Show any intermediary requests/responses'
    complete -c $cmd -s S -l stream -d 'Always stream the response body by line'
    complete -c $cmd -s o -l output -F -d 'Save output to FILE'
    complete -c $cmd -s d -l download -d 'Download a file'
    complete -c $cmd -s c -l continue -d 'Resume an interrupted download'
    complete -c $cmd -s q -l quiet -d 'Do not print to stdout or stderr'


    # Sessions

    complete -c $cmd -l session -F -d 'Create, or reuse and update a session'
    complete -c $cmd -l session-read-only -F -d 'Create or read a session without updating it'


    # Authentication

    complete -c $cmd -s a -l auth -x -d 'Username and password for authentication'
    complete -c $cmd -s A -l auth-type -xa "(__fish_httpie_auth_types)" -d 'The authentication mechanism to be used'
    complete -c $cmd -l ignore-netrc -d 'Ignore credentials from .netrc'


    # Network

    complete -c $cmd -l offline -d 'Build the request and print it but don\'t actually send it'
    complete -c $cmd -l proxy -x -d 'String mapping protocol to the URL of the proxy'
    complete -c $cmd -s F -l follow -d 'Follow 30x Location redirects'
    complete -c $cmd -l max-redirects -x -d 'Set maximum number of redirects'
    complete -c $cmd -l max-headers -x -d 'Maximum number of response headers to be read before giving up'
    complete -c $cmd -l timeout -x -d 'Connection timeout in seconds'
    complete -c $cmd -l check-status -d 'Error with non-200 HTTP status code'
    complete -c $cmd -l path-as-is -d 'Bypass dot segment URL squashing'
    complete -c $cmd -l chunked -d 'Enable streaming via chunked transfer encoding'


    # SSL

    complete -c $cmd -l verify -xa "(__fish_http_verify_options)" -d 'Enable/disable cert verification'
    complete -c $cmd -l ssl -x -d 'Desired protocol version to use'
    complete -c $cmd -l ciphers -x -d 'String in the OpenSSL cipher list format'
    complete -c $cmd -l cert -F -d 'Client side SSL certificate'
    complete -c $cmd -l cert-key -F -d 'Private key to use with SSL'
    complete -c $cmd -l cert-key-pass -x -d 'Passphrase for the given private key'


    # Troubleshooting

    complete -c $cmd -s I -l ignore-stdin -d 'Do not attempt to read stdin'
    complete -c $cmd -l help -d 'Show help'
    complete -c $cmd -l manual -d 'Show the full manual'
    complete -c $cmd -l version -d 'Show version'
    complete -c $cmd -l traceback -d 'Prints exception traceback should one occur'
    complete -c $cmd -l default-scheme -x -d 'The default scheme to use'
    complete -c $cmd -l debug -d 'Show debugging output'
end
"#;

pub(super) fn generate_completion(output: Option<String>) -> std::io::Result<()> {
	let mut writer: Box<dyn Write> = if let Some(path) = output {
		Box::new(std::fs::File::create(path)?)
	} else {
		Box::new(std::io::stdout())
	};

	writer.write_all(FISH_COMPLETE_TEMPLATE.as_bytes())?;
	writer.flush()?;
	Ok(())
}
