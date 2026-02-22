version    = 1
provider   = "github"
categories = ["scm", "issues"]

command "search_issues" {
  title       = "Search Issues"
  summary     = "Search GitHub issues by query string"
  description = "Search issues and pull requests using GitHub's issue search API."
  categories  = ["search", "issues"]

  annotations {
    mode    = "read"
    secrets = ["github.token"]
  }

  param "query" {
    type     = "string"
    required = true
  }

  param "per_page" {
    type     = "integer"
    required = false
    default  = 20
  }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "https://api.github.com/search/issues"

    auth {
      kind   = "bearer"
      secret = "github.token"
    }

    query = {
      q        = "{{ args.query }}"
      per_page = "{{ args.per_page }}"
    }
  }

  result {
    decode = "json"

    extract {
      json_pointer = "/"
    }

    output = "Found {{ result.total_count }} issues."
  }
}

command "create_issue" {
  title       = "Create Issue"
  summary     = "Create a new issue in a repository"
  description = "Create a GitHub issue in the target repository using the REST API."
  categories  = ["write", "issues"]

  annotations {
    mode    = "write"
    secrets = ["github.token"]
  }

  param "owner" {
    type     = "string"
    required = true
  }

  param "repo" {
    type     = "string"
    required = true
  }

  param "title" {
    type     = "string"
    required = true
  }

  param "body" {
    type     = "string"
    required = false
    default  = ""
  }

  operation {
    protocol = "http"
    method   = "POST"
    url      = "https://api.github.com/repos/{{ args.owner }}/{{ args.repo }}/issues"

    auth {
      kind   = "bearer"
      secret = "github.token"
    }

    body {
      kind = "json"
      value = {
        title = "{{ args.title }}"
        body  = "{{ args.body }}"
      }
    }
  }

  result {
    decode = "json"

    extract {
      json_pointer = "/"
    }

    output = "Created issue: {{ result.html_url }}"
  }
}
