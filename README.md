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
  -r, --readonly             Should connection be readonly? [env: MONGODB_READONLY=]
  -n, --noauth               Don't require login tokens [env: MONGODB_NOAUTH=]
  -j, --jwks <JWKS>          JWKS URL [env: MONGODB_JWKS_URL=]
  -a, --audience <AUDIENCE>  JWKS Audience [env: MONGODB_JWKS_AUDIENCE=]
  -h, --help                 Print help
  -V, --version              Print version
```

### Authentication

This API can use any JWKS endpoint to authorize tokens, based on authorized scopes. Tokens will need to have an authorized cluster scope, as well as at least one role scope. Scope format is shown below:

#### Cluster Access

Format: `mongodb.cluster:{{ replicaset }}:allow`

Examples:
```
mongodb.cluster.onprem-dev-cluster01:allow
mongodb.cluster.atlas-11czv4-shard-0:allow
```

#### Admin Roles

Format: `mongodb.role.admin:{{ role }}`  

Roles in increasing order of access:  
- clustermonitor  
- readanydatabase  
- readwriteanydatabase  
- dbadminanydatabase  
- clusteradmin  

Examples:
```
mongodb.role.admin:clustermonitor
mongodb.role.admin:readanydatabase
mongodb.role.admin:readwriteanydatabase
mongodb.role.admin:dbadminanydatabase
mongodb.role.admin:clusteradmin
```

#### Database Specific Roles

Format: `mongodb.role.{{ database }}:{{ role }}`  

Roles in increasing order of access:  
- read  
- readwrite  
- dbadmin  

Examples:
```
mongodb.role.dds_posts:dbadmin
mongodb.role.dds_posts:readwrite
mongodb.role.dds_posts:read
```

## API References

### User
```
# Get user's token roles
GET /user/roles
```

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
GET /:db/collection/:coll/_count

# Get collection indexes
GET /:db/collection/:coll/_indexes[?simple]

# Get collection index stats
GET /:db/collection/:coll/_index_stats

# Get collection stats
GET /:db/collection/:coll/_stats

# Get most recent doc
GET /:db/collection/:coll/_find_one

# Find a document
POST /:db/collection/:coll/_find_one[?format=json|ejson]
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
GET /:db/collection/:coll
GET /:db/collection/:coll/_find

# Find multiple documents
POST /:db/collection/:coll/_find[?format=json|ejson]
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
POST /:db/collection/:coll/_aggregate[?format=json|ejson] 
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
POST /:db/collection/:coll/_distinct
{
  "field_name": String,
  "filter": {},
  "options": {}
}

# Watch collection for changes
GET /:db/collection/:coll/_watch

# Watch collection for changes, with pipeline
POST /:db/collection/:coll/_watch
{
  "pipeline": [{}],
  "options": {
    "full_document": String,
    "full_document_before_change": String,
    "resume_after": String,
    "start_at_operation_time": Timestamp,
    "start_after": String,
    "max_await_time": Duration,
    "batch_size": u32,
    "collation": {},
    "read_concern": String,
    "selection_criteria": {},
    "comment": String,
  }
}
```

### Collection CRUD Operations
```
# Create index
POST /:db/collection/:coll/_indexes
{
  "keys": {}
  "options": {
    "background": bool,
    "expire_after": u64,
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

# Delete index
DELETE /:db/collection/:coll/_indexes/:index

# Insert one doc
POST /:db/collection/:coll/_insert[bypass_document_validation=bool, w=string, n=u32, w_timeout=u32, journal=bool, comment=string]
{}

# Insert one doc
POST /:db/collection/:coll/_insert[bypass_document_validation=bool, ordered=bool, w=string, n=u32, w_timeout=u32, journal=bool, comment=string]
[{}]

# Delete one document
POST /:db/collection/:coll/_delete_one
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
POST /:db/collection/:coll/_delete_many
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

# Update one document
POST /:db/collection/:coll/_update_one
{
  "filter": {},
  "update": {},
  "options": {
    "array_filters": [{}],
    "bypass_document_validation": bool,
    "upsert": bool,
    "collation": {},
    "hint": String,
    "write_concern": String,
    "let_vars": {},
    "comment": String
  }
}

# Update many documents
POST /:db/collection/:coll/_update
{
  "filter": {},
  "update": {},
  "options": {
    "array_filters": [{}],
    "bypass_document_validation": bool,
    "upsert": bool,
    "collation": {},
    "hint": String,
    "write_concern": String,
    "let_vars": {},
    "comment": String
  }
  
# Get database roles
GET /:db/_roles

# Get single database role
GET /:db/_roles/:role

# Create new database role
POST /:db/_roles
{
  "name": String,
  "privileges": [{
    "resource": String,
    "actions": [String]
  }],
  "roles": [{
    "db": String,
    "role": String
  }]
}

# Delete role
DELETE /:db/_roles/:role
```

### Future
``` 
# ReplaceOne
POST /:db/collection/:coll/_replace_one
{
  "filter": {},
  "replacement": {},
  "upsert": bool
}
```
