# mongodb-rest-rs

Simple REST API frontend for MongoDB Replicasets. 

Work in progress, please read src/main.rs to view available API paths and methods.

### Usage
```
Usage: mongodb-rest [OPTIONS] --uri <URI>

Options:
  -p, --port <PORT>  Port to listen on [env: API_PORT=] [default: 8080]
  -u, --uri <URI>    Default connection uri [env: MONGODB_URI=]
  -r, --readonly     Should connection be readonly? [env: MONGODB_READONLY=]
  -h, --help         Print help
  -V, --version      Print version
```

### Endpoints
```
# Get replicaset status
GET /_cat/status

# Get latest logs
GET /_cat/log

# Get current operations
GET /_cat/ops

# Get replicaset stats
GET /_cat/stats

# Get databases
GET /_cat/dbs

# Get collection stats
GET /_cat/top

# Get current connection info
GET /_cat/conn

# Get connection pool info
GET /_cat/pool

# Get database stats
GET /:db/_stats

# Get database collections
GET /:db/_collections

# Get collection document counts
GET /:db/:coll/_count

# Get collection indexes
GET /:db/:coll/_indexes

# Get collection index stats
GET /:db/:coll/_index_stats

# Get collection stats
GET /:db/:coll/_stats

# Find a document
POST /:db/:coll/_find_one
{
  "filter": {},
  "projection": Option<{}>,
}

# Find multiple documents
POST /:db/:coll/_find
{
  "filter": {},
  "projection": Option<{}>,
  "sort": Option<{}>,
  "limit": Option<u64>,
  "skip": Option<u64>,
  "explain": Option<"queryPlanner | executionStats | allPlansExecution">
}

# Aggregation
POST /:db/:coll/_aggregate 
{
  "pipeline": [{}]
  "explain": Option<"queryPlanner | executionStats | allPlansExecution">
}
```

### Future
``` 
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

# DeleteOne/Delete
POST /:db/:coll/_delete
{
  "filter": {}
}
```
