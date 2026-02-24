version = 1
provider = "test_external_secret"

command "ping" {
  title       = "Ping"
  summary     = "Test external secret"
  description = "Tests that external secret references pass validation"

  annotations {
    mode    = "read"
    secrets = ["op://vault/item/token"]
  }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "https://api.example.com/ping"

    auth {
      kind   = "bearer"
      secret = "op://vault/item/token"
    }
  }

  result {
    output = "{{ response.body }}"
  }
}
