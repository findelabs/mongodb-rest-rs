# mongodb-rest-rs

Simple REST API frontend for MongoDB Replicasets. 

Work in progress, please read src/main.rs to view available API paths and methods.

### Usage
```
Usage: mongodb-rest-rs [OPTIONS] --uri <URI>

Options:
  -p, --port <PORT>          Port to listen on [env: API_PORT=] [default: 8080]
  -u, --uri <URI>            Default connection uri [env: MONGODB_URI=]
  -U, --username <USERNAME>  MongoDB username [env: MONGODB_USERNAME=]
  -P, --password <PASSWORD>  MongoDB username password [env: MONGODB_PASSWORD=]
  -h, --help                 Print help
  -V, --version              Print version
```

## API References

### Replicaset
```
# Get replicaset status
GET /rs/status

# Get latest logs
GET /rs/log

# Get current operations
GET /rs/ops

# Get replicaset stats
GET /rs/stats

# Get databases
GET /rs/dbs

# Get collection stats
GET /rs/top

# Get current connection info
GET /rs/conn

# Get connection pool info
GET /rs/pool
```

### Database
``` 
# Get database stats
GET /:db/_stats

# Get database collections
GET /:db

# Get collection document count
GET /:db/:coll/_count

# Get collection indexes
GET /:db/:coll/_indexes[?simple]

# Get collection index stats
GET /:db/:coll/_index_stats

# Get collection stats
GET /:db/:coll/_stats

# Get most recent doc
GET /:db/:coll/_find_one

# Find a document
POST /:db/:coll/_find_one[?format=json|ejson]
{
  "filter": {},
  "options": {
    "allow_partial_results": bool,
    "collation": {},
    "comment": String,
    "hint": {},
    "max": {},
    "max_scan": u64,
    "max_time": {},
    "min": {},
    "projection": {},
    "read_concern": String,
    "return_key": bool,
    "selection_criteria": String,
    "show_record_id": bool,
    "skip": u64,
    "sort": {},
    "let_vars": {}
  }
}

# Get ten most recent docs
GET /:db/:coll
GET /:db/:coll/_find

# Find multiple documents
POST /:db/:coll/_find[?format=json|ejson]
{
  "filter": {},
  "options": {
    "allow_disk_use": bool,
    "allow_partial_results": bool,
    "batch_size": u32,
    "comment": String,
    "cursor_type": String,
    "hint": {},
    "limit": i64,
    "max": {},
    "max_await_time": {},
    "max_scan": u64,
    "max_time": {},
    "min": {},
    "no_cursor_timeout": bool,
    "projection": {},
    "read_concern": String,
    "return_key": bool,
    "selection_criteria": String,
    "show_record_id": bool,
    "skip": u64,
    "sort": {},
    "collation": {},
    "let_vars": {}
  }
}

# Aggregation
POST /:db/:coll/_aggregate[?simple] 
{
  "pipeline": [Document],
  "options": Option<Document>,
  "explain": Option<"queryPlanner | executionStats | allPlansExecution">
}

# Create index
POST /:db/:coll/_index 
{
  "keys": {}
  "options": {
    unique: Option<bool>,
    name: Option<String>,
    partial_filter_expression: Option<Document>,
    sparse: Option<bool>,
    expire_after: Option<Duration>,
    hidden: Option<bool>,
    collation: Option<Collation>,
    weights: Option<Document>,
    default_language: Option<String>,
    language_override: Option<String>,
    text_index_version: Option<TextIndexVersion>
  }
}
```

### Future
``` 
# Find query, show simple results
GET /:id/:coll/_find?simple

# Delete index
DELETE /:db/:coll/_index?name=<index name>

# Insert
POST /:db/:coll/_insert
{}

# Update/UpdateOne
POST /:db/:coll/_update
{
  "filter": {},
  "update": {},
  "upsert": bool
}

# ReplaceOne
POST /:db/:coll/_replace_one
{
  "filter": {},
  "replacement": {},
  "upsert": bool
}

# Distinct
POST /:db/:coll/_distinct
{
  "field_name": String,
  "filter": {},
  "options": DistinctOptions
}

# DeleteOne/Delete
POST /:db/:coll/_delete
{
  "filter": {}
}
```
