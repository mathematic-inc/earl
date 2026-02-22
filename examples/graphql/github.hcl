version = 1
provider = "github_graphql"
categories = ["scm", "graphql"]

command "viewer" {
  title       = "Current user"
  summary     = "Get the authenticated user's profile"
  description = "Fetch the login, name, and bio of the currently authenticated GitHub user via GraphQL."
  categories  = ["profile"]

  annotations {
    mode    = "read"
    secrets = ["github.token"]
  }

  operation {
    protocol = "graphql"
    url      = "https://api.github.com/graphql"

    auth {
      kind   = "bearer"
      secret = "github.token"
    }

    graphql {
      query = <<-EOT
        {
          viewer {
            login
            name
            bio
            company
            location
          }
        }
      EOT
    }
  }

  result {
    decode = "json"
    extract {
      json_pointer = "/data/viewer"
    }
    output = "{{ result.login }} ({{ result.name }})"
  }
}

command "repo_issues" {
  title       = "Repository issues"
  summary     = "List recent issues for a repository"
  description = "Fetch the most recent open issues for a GitHub repository using the GraphQL API."
  categories  = ["issues"]

  annotations {
    mode    = "read"
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

  param "count" {
    type        = "integer"
    required    = false
    default     = 10
    description = "Number of issues to fetch"
  }

  operation {
    protocol = "graphql"
    url      = "https://api.github.com/graphql"

    auth {
      kind   = "bearer"
      secret = "github.token"
    }

    graphql {
      query = <<-EOT
        query RepoIssues($owner: String!, $repo: String!, $count: Int!) {
          repository(owner: $owner, name: $repo) {
            issues(first: $count, orderBy: {field: CREATED_AT, direction: DESC}, states: OPEN) {
              totalCount
              nodes {
                number
                title
                author { login }
                createdAt
                labels(first: 5) {
                  nodes { name }
                }
              }
            }
          }
        }
      EOT
      variables = {
        owner = "{{ args.owner }}"
        repo  = "{{ args.repo }}"
        count = "{{ args.count }}"
      }
    }
  }

  result {
    decode = "json"
    extract {
      json_pointer = "/data/repository/issues"
    }
    output = "{{ result.totalCount }} open issues (showing {{ result.nodes | length }})"
  }
}

command "search_repos" {
  title       = "Search repositories"
  summary     = "Search GitHub repositories by query"
  description = "Search for repositories using GitHub's GraphQL search API with star count and description."
  categories  = ["search"]

  annotations {
    mode    = "read"
    secrets = ["github.token"]
  }

  param "query" {
    type        = "string"
    required    = true
    description = "Search query (e.g. 'language:rust stars:>100')"
  }

  param "count" {
    type        = "integer"
    required    = false
    default     = 5
    description = "Number of results to return"
  }

  operation {
    protocol = "graphql"
    url      = "https://api.github.com/graphql"

    auth {
      kind   = "bearer"
      secret = "github.token"
    }

    graphql {
      query = <<-EOT
        query SearchRepos($query: String!, $count: Int!) {
          search(query: $query, type: REPOSITORY, first: $count) {
            repositoryCount
            nodes {
              ... on Repository {
                nameWithOwner
                description
                stargazerCount
                primaryLanguage { name }
                updatedAt
              }
            }
          }
        }
      EOT
      variables = {
        query = "{{ args.query }}"
        count = "{{ args.count }}"
      }
    }
  }

  result {
    decode = "json"
    extract {
      json_pointer = "/data/search"
    }
    output = "{{ result.repositoryCount }} repositories found (showing {{ result.nodes | length }})"
  }
}
