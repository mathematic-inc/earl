version = 1
provider = "analytics"
categories = ["analytics", "sql"]

command "recent_orders" {
  title       = "Recent orders"
  summary     = "Fetches recent orders from the database"
  description = "Runs a read-only SQL query to fetch the most recent orders, sorted by creation date."
  categories  = ["analytics"]

  annotations {
    mode    = "read"
    secrets = ["analytics.database_url"]
  }

  param "limit" {
    type        = "integer"
    required    = false
    default     = 10
    description = "Maximum number of orders to return"
  }

  operation {
    protocol = "sql"

    sql {
      connection_secret = "analytics.database_url"
      query             = "SELECT id, customer, total FROM orders ORDER BY created_at DESC LIMIT ?"
      params            = ["{{ args.limit }}"]
      sandbox {
        read_only = true
        max_rows  = 100
      }
    }
  }

  result {
    decode = "json"
    output = "Found {{ result | length }} orders"
  }
}

command "count_by_status" {
  title       = "Count by status"
  summary     = "Counts records grouped by status"
  description = "Runs a read-only SQL query to count records in a table grouped by their status column."
  categories  = ["analytics"]

  annotations {
    mode    = "read"
    secrets = ["analytics.database_url"]
  }

  operation {
    protocol = "sql"

    sql {
      connection_secret = "analytics.database_url"
      query             = "SELECT status, COUNT(*) as count FROM orders GROUP BY status ORDER BY count DESC"
      sandbox {
        read_only = true
        max_rows  = 50
      }
    }
  }

  result {
    decode = "json"
    output = "{{ result | length }} status groups found"
  }
}
