version    = 1
provider   = "demo"
categories = ["sample"]

command "ping" {
  title       = "Ping"
  summary     = "Execute a simple ping request"
  description = "Sends a basic ping request and returns the raw response body."

  annotations {
    mode    = "read"
    secrets = []
  }

  param "value" {
    type     = "string"
    required = false
    default  = "hello"
  }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "https://api.example.com/ping"

    query = {
      q = "{{ args.value }}"
    }
  }

  result {
    decode = "text"
    output = "{{ result }}"
  }
}
