# Earl

[![CI](https://github.com/brwse/earl/actions/workflows/ci.yml/badge.svg)](https://github.com/brwse/earl/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/earl)](https://crates.io/crates/earl)
[![docs.rs](https://img.shields.io/docsrs/earl)](https://docs.rs/earl)
[![License: MIT](https://img.shields.io/crates/l/earl)](LICENSE)

AI-safe CLI for AI agents. Earl sits between your agent and external
services, ensuring secrets stay in the OS keychain, requests follow reviewed
templates, and outbound traffic obeys egress rules.

## Why

AI agents with shell or network access can read secrets in plaintext and make
arbitrary API calls. Earl eliminates that risk:

- Agents run `earl call provider.command --param value` instead of raw `curl`
- Secrets are stored in the OS keychain and injected at request time
- Every request is defined by an HCL template that can be reviewed ahead of time
- Outbound traffic is restricted via `[[network.allow]]` egress rules
- Private IPs are blocked to prevent SSRF
- Bash, JavaScript, and SQL execution runs in a sandbox

## Install

```bash
cargo install earl
```

> Requires Node.js and pnpm — Earl embeds a web playground at compile time.

Or use the installer scripts:

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/brwse/earl/main/scripts/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/brwse/earl/main/scripts/install.ps1 | iex
```

### Feature flags

All protocol support is included by default. To install with only specific
protocols:

```bash
cargo install earl --no-default-features --features http,bash
```

| Flag      | Description               | Default |
| --------- | ------------------------- | ------- |
| `http`    | HTTP/REST requests        | yes     |
| `graphql` | GraphQL (requires `http`) | yes     |
| `grpc`    | gRPC calls                | yes     |
| `bash`    | Sandboxed bash execution  | yes     |
| `sql`     | Sandboxed SQL queries     | yes     |

## Quick start

```bash
# Install a template
earl templates install https://raw.githubusercontent.com/brwse/earl/main/examples/github.hcl

# Store a secret in the OS keychain
earl secrets set github.token

# Call a command defined in the template
earl call github.search_issues --query "repo:rust-lang/rust is:issue E-easy" --per_page 5
```

## Templates

Templates are HCL files that define the commands an agent can run. Each command
declares its parameters, the underlying protocol operation, and how to format
the result.

```hcl
version = 1
provider = "github"

command "search_issues" {
  title   = "Search issues"
  summary = "Search GitHub issues with a query string"

  param "query" {
    type     = "string"
    required = true
  }

  param "per_page" {
    type    = "number"
    default = 10
  }

  operation {
    http {
      method = "GET"
      url    = "https://api.github.com/search/issues"
      query  = { q = param.query, per_page = param.per_page }
      headers = {
        Authorization = "Bearer ${secret.github.token}"
      }
    }
  }
}
```

Manage templates with the `earl templates` subcommand:

```bash
earl templates list           # List all available commands
earl templates search "issues" # Semantic search across templates
earl templates validate ./t.hcl # Validate a template file
```

## Secrets

Secrets are stored in the OS keychain (macOS Keychain, Windows Credential
Manager, Linux Secret Service) and referenced in templates as
`${secret.provider.name}`.

```bash
earl secrets set github.token   # Prompt for value
earl secrets list               # List stored secrets
earl secrets delete github.token
```

## OAuth2

Earl manages OAuth2 tokens with automatic refresh:

```bash
earl auth login my-profile
earl auth status
earl auth refresh my-profile
earl auth logout my-profile
```

## MCP server

Earl runs as an [MCP](https://modelcontextprotocol.io) server so agents can
discover and call commands over a standard protocol:

```bash
earl mcp stdio          # stdio transport (default)
earl mcp http           # HTTP transport on 127.0.0.1:8977
earl mcp http --yes     # auto-approve write operations
```

## Web playground

A built-in web UI lets you browse the command catalog and test calls
interactively:

```bash
earl web
```

## Configuration

Earl reads `~/.config/earl/config.toml`. Use it to set network egress rules,
proxy profiles, and sandbox policies.

> **Important:** Restrict write access to the config file and template
> directories, or the security model is undermined.

## CLI reference

| Command                   | Description                                         |
| ------------------------- | --------------------------------------------------- |
| `earl call <command>`     | Execute a template command                          |
| `earl templates <sub>`    | List, search, validate, install, generate templates |
| `earl secrets <sub>`      | Manage keychain secrets                             |
| `earl auth <sub>`         | OAuth2 profile management                           |
| `earl mcp <transport>`    | Start the MCP server                                |
| `earl web`                | Launch the web playground                           |
| `earl doctor`             | Run system diagnostics                              |
| `earl completion <shell>` | Generate shell completions                          |

## Project structure

```
crates/
  earl-core/             Shared types and traits
  earl-protocol-http/    HTTP/GraphQL protocol
  earl-protocol-grpc/    gRPC protocol
  earl-protocol-bash/    Sandboxed bash execution
  earl-protocol-sql/     Sandboxed SQL execution
src/                     Main CLI application
web/                     React/Vite playground (embedded at build)
site/                    Next.js documentation site
```

## Documentation

Full docs at [brwse.github.io/earl/docs](https://brwse.github.io/earl/docs/):
[Quick Start](https://brwse.github.io/earl/docs/quick-start) ·
[Security Model](https://brwse.github.io/earl/docs/security) ·
[Templates](https://brwse.github.io/earl/docs/templates) ·
[Configuration](https://brwse.github.io/earl/docs/configuration) ·
[MCP Integration](https://brwse.github.io/earl/docs/mcp) ·
[CLI Reference](https://brwse.github.io/earl/docs/commands)

## License

[MIT](LICENSE)
