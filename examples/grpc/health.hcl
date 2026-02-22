version = 1
provider = "grpc_health"
categories = ["grpc", "health"]

command "check" {
  title       = "Health check"
  summary     = "Check the health of a gRPC service"
  description = "Calls the standard gRPC health checking protocol to determine if a service is healthy."
  categories  = ["health"]

  annotations {
    mode = "read"
  }

  param "url" {
    type        = "string"
    required    = true
    description = "gRPC server URL (e.g. http://api.example.com:50051)"
  }

  param "service" {
    type        = "string"
    required    = false
    default     = ""
    description = "Service name to check (empty string checks the server overall)"
  }

  operation {
    protocol = "grpc"
    url      = "{{ args.url }}"

    grpc {
      service = "grpc.health.v1.Health"
      method  = "Check"
      body = {
        service = "{{ args.service }}"
      }
    }

    transport {
      timeout_ms = 5000
    }
  }

  result {
    decode = "json"
    output = "{{ result.status }}"
  }
}

command "check_with_descriptor" {
  title       = "Health check (offline)"
  summary     = "Health check using a local descriptor set"
  description = "Calls the gRPC health check using a pre-compiled descriptor set file instead of server reflection. Useful when the server does not expose the reflection service."
  categories  = ["health"]

  annotations {
    mode = "read"
  }

  param "url" {
    type        = "string"
    required    = true
    description = "gRPC server URL (e.g. http://api.example.com:50051)"
  }

  param "service" {
    type        = "string"
    required    = false
    default     = ""
    description = "Service name to check"
  }

  param "descriptor" {
    type        = "string"
    required    = true
    description = "Path to the .fds.bin descriptor set file"
  }

  operation {
    protocol = "grpc"
    url      = "{{ args.url }}"

    grpc {
      service             = "grpc.health.v1.Health"
      method              = "Check"
      descriptor_set_file = "{{ args.descriptor }}"
      body = {
        service = "{{ args.service }}"
      }
    }

    transport {
      timeout_ms = 5000
    }
  }

  result {
    decode = "json"
    output = "{{ result.status }}"
  }
}
