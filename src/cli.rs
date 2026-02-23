use std::net::SocketAddr;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "earl",
    version,
    about = "AI-safe CLI for executing provider commands"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Execute a provider command (e.g. earl call provider.command --param value).
    Call(CallArgs),
    /// Work with templates.
    Templates(TemplateArgs),
    /// Manage secrets in secure keychain.
    Secrets(SecretsArgs),
    /// Manage OAuth2 authentication profiles.
    Auth(AuthArgs),
    /// Run an MCP server over stdio or HTTP.
    Mcp(McpArgs),
    /// Diagnose configuration and setup issues.
    Doctor(DoctorArgs),
    /// Launch local web docs + playground.
    Web(WebArgs),
    /// Generate shell completion scripts.
    Completion(CompletionArgs),
}

#[derive(Debug, Args)]
pub struct CallArgs {
    /// Command in the form provider.command (e.g. github.search_issues --query "test")
    pub command: String,
    /// Arguments as --key value pairs.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
    /// Auto-approve write-mode templates.
    #[arg(long)]
    pub yes: bool,
    /// Print structured JSON output instead of rendered text.
    #[arg(long)]
    pub json: bool,
    /// Active environment (e.g. --env staging).
    #[arg(long)]
    pub env: Option<String>,
}

#[derive(Debug, Args)]
pub struct TemplateArgs {
    /// Print structured JSON output.
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: TemplateSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum TemplateSubcommand {
    /// List template commands.
    List(TemplateListArgs),
    /// Semantic search over templates.
    Search(TemplateSearchArgs),
    /// Validate template files.
    Validate,
    /// Import a template file from a local path or direct HTTP(S) URL.
    Import(TemplateImportArgs),
    /// Generate a template by delegating to a coding CLI.
    Generate(TemplateGenerateArgs),
}

#[derive(Debug, Args)]
pub struct TemplateListArgs {
    #[arg(long)]
    pub category: Option<String>,
    #[arg(long)]
    pub mode: Option<TemplateMode>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TemplateMode {
    #[value(name = "read")]
    Read,
    #[value(name = "write")]
    Write,
}

#[derive(Debug, Args)]
pub struct TemplateSearchArgs {
    pub query: String,
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct TemplateImportArgs {
    /// Template import source: local file path or direct HTTP(S) URL to a .hcl file.
    pub source_ref: String,
    /// Import destination scope.
    #[arg(long, value_enum, default_value_t = TemplateImportScope::Local)]
    pub scope: TemplateImportScope,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TemplateImportScope {
    #[value(name = "local")]
    Local,
    #[value(name = "global")]
    Global,
}

#[derive(Debug, Args)]
pub struct TemplateGenerateArgs {
    /// Coding CLI invocation after `--` (example: `-- claude --dangerously-skip-permissions`).
    #[arg(
        required = true,
        num_args = 1..,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub command: Vec<String>,
}

#[derive(Debug, Args)]
pub struct SecretsArgs {
    #[command(subcommand)]
    pub command: SecretsSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum SecretsSubcommand {
    /// Store a secret value.
    Set(SecretSetArgs),
    /// Show metadata for a secret key.
    Get(SecretGetArgs),
    /// List known secret keys.
    List,
    /// Delete a secret.
    Delete(SecretDeleteArgs),
}

#[derive(Debug, Args)]
pub struct SecretSetArgs {
    pub key: String,
    /// Read secret value from stdin.
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Debug, Args)]
pub struct SecretGetArgs {
    pub key: String,
}

#[derive(Debug, Args)]
pub struct SecretDeleteArgs {
    pub key: String,
}

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthSubcommand {
    /// Start OAuth login for a profile.
    Login(AuthProfileArgs),
    /// Show current token status.
    Status(AuthProfileArgs),
    /// Force token refresh.
    Refresh(AuthProfileArgs),
    /// Delete token for a profile.
    Logout(AuthProfileArgs),
}

#[derive(Debug, Args)]
pub struct AuthProfileArgs {
    pub profile: String,
}

#[derive(Debug, Args)]
pub struct McpArgs {
    /// Transport to use for the MCP server.
    #[arg(value_enum, default_value_t = McpTransport::Stdio)]
    pub transport: McpTransport,
    /// Listen address used when --transport http is selected.
    #[arg(long, default_value = "127.0.0.1:8977")]
    pub listen: SocketAddr,
    /// MCP serving mode (full catalog or discovery wrappers).
    #[arg(long, value_enum, default_value_t = McpMode::Full)]
    pub mode: McpMode,
    /// Auto-approve write-mode templates for MCP tool calls.
    #[arg(long)]
    pub yes: bool,
    /// Allow unauthenticated access to the MCP HTTP transport (not recommended for production).
    #[arg(long)]
    pub allow_unauthenticated: bool,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Print structured JSON output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct WebArgs {
    /// Listen address for the web playground server.
    #[arg(long, default_value = "127.0.0.1:0")]
    pub listen: SocketAddr,
    /// Do not auto-open the browser.
    #[arg(long)]
    pub no_open: bool,
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    /// Shell to generate completion script for.
    #[arg(value_enum)]
    pub shell: CompletionShell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CompletionShell {
    #[value(name = "bash")]
    Bash,
    #[value(name = "zsh")]
    Zsh,
    #[value(name = "fish")]
    Fish,
    #[value(name = "powershell")]
    PowerShell,
    #[value(name = "elvish")]
    Elvish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum McpTransport {
    #[value(name = "stdio")]
    Stdio,
    #[value(name = "http")]
    Http,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum McpMode {
    #[value(name = "full")]
    Full,
    #[value(name = "discovery")]
    Discovery,
}
