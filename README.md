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

### Future
```
# Post
{}

# FindOne
{
  "filter": {},
  "projection": {}
}

# Find
{
  "filter": {},
  "projection": {},
  "sort": {},
  "limit": u64,
  "skip": u64
}

# Update/UpdateOne
{
  "filter": {},
  "update": {},
  "upsert": bool
}

# ReplaceOne
{
  "filter": {},
  "replacement": {},
  "upsert": bool
}

# DeleteOne/Delete
{
  "filter": {}
}

# Aggregate
{
  "pipeline": [{}]
}
```
