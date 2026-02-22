version = 1
provider = "github"
categories = ["scm", "issues"]

command "search_issues" {
  title       = "Search issues"
  summary     = "Search GitHub issues by query string"
  description = "Search issues and pull requests across repositories using GitHub's search API."
  categories  = ["search", "issues"]

  annotations {
    mode    = "read"
    secrets = ["github.token"]
  }

  param "query" {
    type        = "string"
    required    = true
    description = "Search query (e.g. 'repo:owner/repo is:open label:bug')"
  }

  param "per_page" {
    type        = "integer"
    required    = false
    default     = 20
    description = "Number of results per page"
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

    headers = {
      Accept = "application/vnd.github+json"
    }
  }

  result {
    decode = "json"
    output = "Found {{ result.total_count }} issues."
  }
}

command "create_issue" {
  title       = "Create issue"
  summary     = "Create a new issue in a repository"
  description = "Create a GitHub issue in the target repository."
  categories  = ["write", "issues"]

  annotations {
    mode    = "write"
    secrets = ["github.token"]
  }

  param "owner" {
    type        = "string"
    required    = true
    description = "Repository owner"
  }

  param "repo" {
    type        = "string"
    required    = true
    description = "Repository name"
  }

  param "title" {
    type        = "string"
    required    = true
    description = "Issue title"
  }

  param "body" {
    type        = "string"
    required    = false
    default     = ""
    description = "Issue body (Markdown)"
  }

  operation {
    protocol = "http"
    method   = "POST"
    url      = "https://api.github.com/repos/{{ args.owner }}/{{ args.repo }}/issues"

    auth {
      kind   = "bearer"
      secret = "github.token"
    }

    headers = {
      Accept = "application/vnd.github+json"
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
    output = "Created issue #{{ result.number }}: {{ result.html_url }}"
  }
}

command "list_repos" {
  title       = "List repositories"
  summary     = "List repositories for a user or organization"
  description = "Fetch public repositories for the specified owner, sorted by most recently updated."
  categories  = ["search"]

  annotations {
    mode    = "read"
    secrets = ["github.token"]
  }

  param "owner" {
    type        = "string"
    required    = true
    description = "GitHub username or organization"
  }

  param "per_page" {
    type        = "integer"
    required    = false
    default     = 10
    description = "Number of repositories to return"
  }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "https://api.github.com/users/{{ args.owner }}/repos"

    auth {
      kind   = "bearer"
      secret = "github.token"
    }

    query = {
      sort     = "updated"
      per_page = "{{ args.per_page }}"
    }

    headers = {
      Accept = "application/vnd.github+json"
    }
  }

  result {
    decode = "json"
    output = "{{ result | length }} repositories"
  }
}
