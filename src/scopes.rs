use std::collections::HashMap;

#[derive(Clone)]
pub struct AuthorizeScope {
    noauth: bool,
    sub: String,
    roles: HashMap<String, Vec<String>>,
}

use crate::error::Error as RestError;

// Roles in admin db
const ADMIN_READ_ROLES: &'static [&'static str] = &[
    "readanydatabase",
    "readwriteanydatabase",
    "dbadminanydatabase",
    "clusteradmin",
];
const ADMIN_WRITE_ROLES: &'static [&'static str] =
    &["readwriteanydatabase", "dbadminanydatabase", "clusteradmin"];
const ADMIN_DBADMIN_ROLES: &'static [&'static str] = &["dbadminanydatabase", "clusteradmin"];
const ADMIN_MONITOR_ROLES: &'static [&'static str] = &["clusteradmin", "clustermonitor"];

// Roles for specific db's
const DB_READ_ROLES: &'static [&'static str] = &["read", "readwrite", "dbadmin"];
const DB_WRITE_ROLES: &'static [&'static str] = &["readwrite", "dbadmin"];
const DB_DBADMIN_ROLES: &'static [&'static str] = &["dbadmin"];
const DB_MONITOR_ROLES: &'static [&'static str] = &["read", "readwrite", "dbadmin"];

impl AuthorizeScope {
    pub fn default() -> Self {
        AuthorizeScope {
            noauth: true,
            sub: "noauth".to_string(),
            roles: HashMap::new(),
        }
    }

    pub fn roles(&self) -> HashMap<String, Vec<String>> {
        self.roles.clone()
    }

    pub fn authorized_dbs(&self) -> Vec<String> {
        // Return all keys, skipping cluster key
        self.roles
            .clone()
            .into_keys()
            .filter(|k| k != &"cluster")
            .collect()
    }

    pub fn new(
        cluster: Option<String>,
        scopes: Vec<String>,
        subject: String,
    ) -> Result<Self, RestError> {
        let replicaset = match cluster {
            Some(c) => c,
            None => {
                log::warn!("\"sub={}, Did not detect replicaset name\"", subject);
                return Err(RestError::UnauthorizedClient);
            }
        };

        let mut clusters: Vec<String> = Vec::new();
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        for scope in scopes {
            log::debug!("\"sub={}, Extracting scope {}", subject, scope);

            let colon_split: Vec<&str> = scope.split(':').collect();
            let action = match colon_split.get(1) {
                Some(i) => i.to_string(),
                None => continue,
            };

            let period_split: Vec<&str> = colon_split[0].split('.').collect();

            // This should always be mongodb
            match period_split.get(0) {
                Some(i) => {
                    // Skip a scope if tech is not mongodb
                    if i != &"mongodb" {
                        log::debug!(
                            "\"sub={}, Scope technology does not equal mongodb, skipping",
                            subject
                        );
                        continue;
                    } else {
                        i
                    }
                }
                None => continue,
            };

            let kind = match period_split.get(1) {
                Some(i) => i.to_string(),
                None => continue,
            };

            let value = match period_split.get(2) {
                Some(i) => i.to_string(),
                None => continue,
            };

            // Check for either a role or cluster
            if kind == "role" {
                // Generate HashMap of database -> [role]
                match map.get_mut(&value) {
                    Some(v) => {
                        log::debug!(
                            "\"sub={}, Appending role {} role {}\"",
                            subject,
                            action,
                            value
                        );
                        v.push(action);
                    }
                    None => {
                        log::debug!("\"sub={}, Adding role {} for {}\"", subject, action, value);
                        let mut vec = Vec::new();
                        vec.push(action);
                        map.insert(value, vec);
                    }
                }
            } else if kind == "cluster" && action == "allow" {
                if !clusters.contains(&value) {
                    log::debug!(
                        "\"sub={}, Adding cluster access for replicaset {}\"",
                        subject,
                        value
                    );
                    clusters.push(value)
                }
            }
        }

        // Ensure that client has access to current cluster
        if !clusters.contains(&replicaset) {
            log::warn!(
                "\"sub={}, Did not find {} in authorized clusters\"",
                subject,
                replicaset
            );
            return Err(RestError::UnauthorizedClient);
        }

        log::debug!("\"sub={}, Scope map: {:?}\"", subject, map);
        Ok(AuthorizeScope {
            noauth: false,
            sub: subject,
            roles: map,
        })
    }

    pub fn monitor(&self, db: &str) -> Result<(), RestError> {
        // return early if noauth is true
        if self.noauth {
            log::debug!("\"sub={}, No cluster auth, exiting monitor fn\"", self.sub);
            return Ok(());
        }

        // Check if client has admin db rights to monitor any database
        if let Some(roles) = self.roles.get("admin") {
            for role in roles {
                if ADMIN_MONITOR_ROLES.contains(&role.as_str()) {
                    log::debug!("\"sub={}, db=admin, role=monitor, action=allow\"", self.sub);
                    return Ok(());
                }
            }
        };

        // Check if client has monitor rights to requested db
        if let Some(roles) = self.roles.get(db) {
            for role in roles {
                if DB_MONITOR_ROLES.contains(&role.as_str()) {
                    log::debug!(
                        "\"sub={}, db={}, role=monitor, action=allow\"",
                        self.sub,
                        db
                    );
                    return Ok(());
                }
            }
        };

        // If we got here, there were no matched roles
        log::warn!(
            "\"sub={}, db={}, role=monitor, action=reject\"",
            self.sub,
            db
        );
        Err(RestError::UnauthorizedClient)
    }

    pub fn write(&self, db: &str) -> Result<(), RestError> {
        // return early if noauth is true
        if self.noauth {
            log::debug!("\"sub={}, No cluster auth, exiting write fn\"", self.sub);
            return Ok(());
        }

        // Check if client has admin db rights to write any database
        if let Some(roles) = self.roles.get("admin") {
            for role in roles {
                if ADMIN_WRITE_ROLES.contains(&role.as_str()) {
                    log::debug!("\"sub={}, db=admin, role=write, action=allow\"", self.sub);
                    return Ok(());
                }
            }
        };

        // Check if client has write rights to requested db
        if let Some(roles) = self.roles.get(db) {
            for role in roles {
                if DB_WRITE_ROLES.contains(&role.as_str()) {
                    log::debug!("\"sub={}, db={}, role=write, action=allow\"", self.sub, db);
                    return Ok(());
                }
            }
        };

        // If we got here, there were no matched roles
        log::warn!("\"sub={}, db={}, role=write, action=reject\"", self.sub, db);
        Err(RestError::UnauthorizedClient)
    }

    pub fn dbadmin(&self, db: &str) -> Result<(), RestError> {
        // return early if noauth is true
        if self.noauth {
            log::debug!("\"sub={}, No cluster auth, exiting dbadmin fn\"", self.sub);
            return Ok(());
        }

        // Check if client has admin db rights to admin any database
        if let Some(roles) = self.roles.get("admin") {
            for role in roles {
                if ADMIN_DBADMIN_ROLES.contains(&role.as_str()) {
                    log::debug!("\"sub={}, db=admin, role=dbadmin, action=allow\"", self.sub);
                    return Ok(());
                }
            }
        };

        // Check if client has dbadmin rights to requested db
        if let Some(roles) = self.roles.get(db) {
            for role in roles {
                if DB_DBADMIN_ROLES.contains(&role.as_str()) {
                    log::debug!(
                        "\"sub={}, db={}, role=dbadmin, action=allow\"",
                        self.sub,
                        db
                    );
                    return Ok(());
                }
            }
        };

        // If we got here, there were no matched roles
        log::warn!(
            "\"sub={}, db={}, role=dbadmin, action=reject\"",
            self.sub,
            db
        );
        Err(RestError::UnauthorizedClient)
    }

    pub fn read(&self, db: &str) -> Result<(), RestError> {
        // return early if noauth is true
        if self.noauth {
            log::debug!("\"sub={}, No cluster auth, exiting read fn\"", self.sub);
            return Ok(());
        }

        // Check if client has admin db rights to read any database
        if let Some(roles) = self.roles.get("admin") {
            for role in roles {
                if ADMIN_READ_ROLES.contains(&role.as_str()) {
                    log::debug!("\"sub={}, db=admin, role=read, action=allow\"", self.sub);
                    return Ok(());
                }
            }
        };

        // Check if client has read rights to requested db
        if let Some(roles) = self.roles.get(db) {
            for role in roles {
                if DB_READ_ROLES.contains(&role.as_str()) {
                    log::debug!("\"sub={}, db={}, role=read, action=allow\"", self.sub, db);
                    return Ok(());
                }
            }
        };

        // If we got here, there were no matched roles
        log::warn!("\"sub={}, db={}, role=read, action=reject\"", self.sub, db);
        Err(RestError::UnauthorizedClient)
    }
}
