use std::collections::HashMap;

#[derive(Clone)]
pub struct AuthorizeScope {
    noauth: bool,
    scopes: HashMap<String, Vec<String>>
}

use crate::error::Error as RestError;

// Roles in admin db
const ADMIN_READ_ROLES: &'static [&'static str] = &["readanydatabase", "readwriteanydatabase", "dbadminanydatabase", "clusteradmin"];
const ADMIN_WRITE_ROLES: &'static [&'static str] = &["readwriteanydatabase", "dbadminanydatabase", "clusteradmin"];
const ADMIN_MONITOR_ROLES: &'static [&'static str] = &["clusteradmin", "clustermonitor"];

// Roles for specific db's
const DB_READ_ROLES: &'static [&'static str] = &["read", "readwrite", "dbadmin"];
const DB_WRITE_ROLES: &'static [&'static str] = &["readwrite", "dbadmin"];
const DB_MONITOR_ROLES: &'static [&'static str] = &["read", "readwrite", "dbadmin"];

impl AuthorizeScope {
    pub fn default() -> Self {
        AuthorizeScope { noauth: true, scopes: HashMap::new() }
    }

    pub fn authorized_dbs(&self) -> Vec<String> {
        // Return all keys, skipping cluster key
        self.scopes.clone().into_keys().filter(|k| k != &"cluster").collect()
    }

    pub fn new(cluster: Option<String>, scopes: Vec<String>, subject: String) -> Result<Self, RestError> {
        let replicaset = match cluster {
            Some(c) => c,
            None => {
                log::warn!("\"sub={}, Did not detect replicaset name\"", subject);
                return Err(RestError::UnauthorizedClient)
            }
        };

        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        
        for scope in scopes {
            log::debug!("\"sub={}, Extracting scope {}", subject, scope);

            let colon_split: Vec<&str> = scope.split(':').collect();
            let action = match colon_split.get(1) {
                Some(i) => i.to_string(),
                None => continue
            };

            let period_split: Vec<&str> = colon_split[0].split('.').collect();
            let tech = match period_split.get(0) {
                Some(i) => i,
                None => continue
            };
            let database = match period_split.get(1) {
                Some(i) => i.to_string(),
                None => continue
            };

            // Skip a scope if tech is not mongodb
            if tech != &"mongodb" {
                log::debug!("\"sub={}, Scope technology does not equal mongodb, skipping", subject);
                continue
            }

            // Generate HashMap of database -> [role]
            match map.get_mut(&database) {
                Some(v) => {
                    log::debug!("\"sub={}, Appending {} to {}\"", subject, action, database);
                    v.push(action);
                },
                None => {
                    log::debug!("\"sub={}, Creating {} for {}\"", subject, action, database);
                    let mut vec = Vec::new();
                    vec.push(action);
                    map.insert(database, vec);
                }
            }
        }

        // Ensure that client has access to current cluster
        match map.get("cluster") {
            Some(c) => if !c.contains(&replicaset) {
                log::debug!("\"sub={}, Did not find {} in authorized clusters in token\"", subject, replicaset);
                return Err(RestError::UnauthorizedClient)
            },
            None => {
                log::debug!("\"sub={}, Did not find authorized cluster scope in token\"", subject);
                return Err(RestError::UnauthorizedClient)
            }
        };

        log::debug!("\"sub={}, Scope map: {:?}\"", subject, map);

        Ok(AuthorizeScope { noauth: false, scopes: map })
    }

    pub fn monitor(&self, db: &str) -> Result<(), RestError>{
        // return early if noauth is true
        if self.noauth {
            return Ok(())
        }

        // Check if client has admin db rights to monitor any database
        if let Some(roles) = self.scopes.get("admin") {
            for role in roles {
                if ADMIN_MONITOR_ROLES.contains(&role.as_str()) {
                    return Ok(())
                }
            }
        };

        // Check if client has monitor rights to requested db
        if let Some(roles) = self.scopes.get(db) {
            for role in roles {
                if DB_MONITOR_ROLES.contains(&role.as_str()) {
                    return Ok(())
                }
            }
        };

        // If we got here, there were no matched roles
        Err(RestError::UnauthorizedClient)
    }

    pub fn write(&self, db: &str) -> Result<(), RestError>{
        // return early if noauth is true
        if self.noauth {
            return Ok(())
        }

        // Check if client has admin db rights to write any database
        if let Some(roles) = self.scopes.get("admin") {
            for role in roles {
                if ADMIN_WRITE_ROLES.contains(&role.as_str()) {
                    return Ok(())
                }
            }
        };

        // Check if client has write rights to requested db
        if let Some(roles) = self.scopes.get(db) {
            for role in roles {
                if DB_WRITE_ROLES.contains(&role.as_str()) {
                    return Ok(())
                }
            }
        };

        // If we got here, there were no matched roles
        Err(RestError::UnauthorizedClient)
    }

    pub fn read(&self, db: &str) -> Result<(), RestError>{
        // return early if noauth is true
        if self.noauth {
            return Ok(())
        }

        // Check if client has admin db rights to read any database
        if let Some(roles) = self.scopes.get("admin") {
            for role in roles {
                if ADMIN_READ_ROLES.contains(&role.as_str()) {
                    return Ok(())
                }
            }
        };

        // Check if client has read rights to requested db
        if let Some(roles) = self.scopes.get(db) {
            for role in roles {
                if DB_READ_ROLES.contains(&role.as_str()) {
                    return Ok(())
                }
            }
        };

        // If we got here, there were no matched roles
        Err(RestError::UnauthorizedClient)
    }
}
