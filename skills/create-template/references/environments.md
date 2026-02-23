# Environments

Named environments let a single template work with multiple backends (production, staging, dev) by defining variable sets that are selected at runtime with `--env <name>`.

Environments are optional — templates work without them.

## Provider-Level Environments Block

Define at the root of the template file:

```hcl
version = 1
provider = "myapi"

environments {
  default = "production"
  secrets = ["myapi.staging_token", "myapi.prod_token"]

  production {
    base_url  = "https://api.example.com"
    api_token = "{{ secrets.myapi_prod_token }}"
  }

  staging {
    base_url  = "https://staging.example.com"
    api_token = "{{ secrets.myapi_staging_token }}"
  }
}
```

| Field | Required | Description |
| ----- | -------- | ----------- |
| `default` | No | Environment name to use when no `--env` flag is given |
| `secrets` | No | Secret keys that `vars` values may reference via `{{ secrets.* }}` |
| `<name> { ... }` | No | Named environment blocks; key-value pairs become `vars.<key>` |

Environment names must match `[a-zA-Z0-9_-]` and be 1-64 characters.

## Using `vars.*` in Templates

Variables from the active environment are accessible as `vars.<key>`:

```hcl
operation {
  protocol = "http"
  method   = "GET"
  url      = "{{ vars.base_url }}/items"

  auth {
    kind   = "bearer"
    secret = "myapi.token"
  }
}
```

**Important:** `vars` values are rendered with only `secrets` context — you cannot reference `args.*` inside environment variable definitions. Use `args` in the operation/result templates instead.

## Per-Command Environment Overrides

Override the entire operation (and optionally result) for a specific environment:

```hcl
command "sync_data" {
  annotations {
    mode                                 = "write"
    allow_environment_protocol_switching = true
  }

  operation {
    protocol = "http"
    method   = "POST"
    url      = "{{ vars.base_url }}/sync"
  }

  environment "local" {
    operation {
      protocol = "bash"
      bash {
        script = "echo synced locally"
      }
    }
    result {
      decode = "text"
      output = "Local sync: {{ result }}"
    }
  }

  result {
    decode = "text"
    output = "{{ result }}"
  }
}
```

The `result` block inside `environment` is optional — if omitted, the command's default result is used.

## Protocol Switching Guard

If an environment override changes the protocol (e.g., HTTP to Bash), you **must** set:

```hcl
annotations {
  allow_environment_protocol_switching = true
}
```

Without this, validation fails. This prevents accidental sandbox bypass.

## Environment Resolution Order

The active environment is selected in this priority (highest first):

1. `--env <name>` CLI flag
2. `[environments] default` in `~/.config/earl/config.toml`
3. `default` field in the template's `environments` block
4. No active environment — `vars` is an empty map

## When to Use Environments

- **Different API endpoints** per stage (production, staging, dev)
- **Different credentials** per stage
- **Mock/local overrides** for testing (e.g., bash instead of HTTP)
- **Regional configurations** (US, EU endpoints)

If the template only ever hits one endpoint, you don't need environments.
