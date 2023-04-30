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

### Collection Info and Search
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
    "max_time": u32,
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
POST /:db/:coll/_aggregate[?format=json|ejson] 
{
  "pipeline": [{}],
  "options": {
    "allow_disk_use": bool,
    "batch_size": u32,
    "bypass_document_validation": bool,
    "collation": {},
    "comment": String,
    "hint": {},
    "max_await_time": u32,
    "max_time": u32,
    "read_concern": String,
    "selection_criteria": String,
    "write_concern": String,
    "let_vars": {}
}

# Distinct
POST /:db/:coll/_distinct
{
  "field_name": String,
  "filter": {},
  "options": {}
}
```

### Collection CRUD Operations
```
# Create index
POST /:db/:coll/_index 
{
  "keys": {}
  "options": {
    "background": bool,
    "expire_after": u32,
    "name": String,
    "sparse": bool,
    "storage_engine": {},
    "unique": bool,
    "version": u32,
    "default_language": String,
    "language_override": String,
    "text_index_version": u32,
    "weights": {},
    "sphere_2d_index_version": u32,
    "bits": u32,
    "max": f64,
    "min": f64,
    "bucket_size": u32,
    "partial_filter_expression": {},
    "collation": {},
    "wildcard_projection": {},
    "hidden": bool
  }
}

# Insert one doc
POST /:db/:coll/_insert[bypass_document_validation=bool, w=string, n=u32, w_timeout=u32, journal=bool, comment=string]
{}

# Insert one doc
POST /:db/:coll/_insert[bypass_document_validation=bool, ordered=bool, w=string, n=u32, w_timeout=u32, journal=bool, comment=string]
[{}]

# Delete one document
POST /:db/:coll/_delete_one
{
  "filter": {},
  "options": {
    "collation": {},
    "write_concern": {},
    "hint": {},
    "let_vars": {},
    "comment:" String
  }
}

# Delete many documents
POST /:db/:coll/_delete_many
{
  "filter": {},
  "options": {
    "collation": {},
    "write_concern": {},
    "hint": {},
    "let_vars": {},
    "comment:" String
  }
}
```

### Future
``` 
# Delete index
DELETE /:db/:coll/_index?name=<index name>

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


```
