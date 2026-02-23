version = 1
provider = "envtest"

environments {
  default = "production"
  secrets = []
  production {
    base_url = "https://prod.example.com"
    label    = "prod"
  }
  staging {
    base_url = "https://staging.example.com"
    label    = "stg"
  }
}

command "echo_env" {
  title       = "Echo env"
  summary     = "Returns which environment is active"
  description = "Returns the environment label from vars."
  annotations {
    mode    = "read"
    secrets = []
  }
  operation {
    protocol = "bash"
    bash {
      script = "echo {{ vars.label }}"
    }
  }
  result {
    decode = "text"
    output = "{{ result }}"
  }
}

command "override_in_staging" {
  title       = "Override"
  summary     = "Uses a bash script in staging instead of HTTP"
  description = "HTTP GET in production, bash in staging — exercises the protocol-switching guard."
  annotations {
    mode                                 = "read"
    secrets                              = []
    allow_environment_protocol_switching = true
  }
  operation {
    protocol = "http"
    method   = "GET"
    url      = "{{ vars.base_url }}/ping"
  }
  environment "staging" {
    operation {
      protocol = "bash"
      bash {
        script = "echo staging_override"
      }
    }
  }
  result {
    decode = "text"
    output = "{{ result }}"
  }
}
