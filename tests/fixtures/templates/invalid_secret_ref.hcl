version  = 1
provider = "demo"

command "ping" {
  title       = "Ping"
  summary     = "Execute a simple ping request"
  description = "Sends a basic ping request."

  annotations {
    mode    = "read"
    secrets = []
  }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "https://api.example.com/ping"

    auth {
      kind   = "bearer"
      secret = "missing.secret"
    }
  }

  result {
    output = "ok"
  }
}
