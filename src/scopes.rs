use std::collections::HashMap;
use chrono::offset::Utc;
use chrono::{DateTime, TimeZone};
use std::fmt;

use crate::error::Error as RestError;
use crate::auth::Claims;

#[derive(Clone, Debug)]
pub struct AuthorizeScope {
    noauth: bool,
    sub: String,
    exp: DateTime<Utc>,
    jti: String,
    roles: HashMap<String, Vec<String>>,
}


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

impl fmt::Display for AuthorizeScope {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let roles: Vec<String> = self.roles.iter().map(|(k,v)| {
            format!("{}:{}", k, v.join(":"))
        }).collect(); 
        write!(f, "sub={}, jti={}, exp={}, roles={}", self.sub, self.jti, self.exp, roles.join(","))
    }
}


impl AuthorizeScope {
    pub fn default() -> Self {
        AuthorizeScope {
            noauth: true,
            sub: "noauth".to_string(),
            exp: Utc::now(),
            jti: String::new(),
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
        cluster: Option<Vec<String>>,
        claims: Claims,
    ) -> Result<Self, RestError> {
        let replicaset = match cluster {
            Some(c) => c,
            None => {
                log::warn!("\"sub={}, Did not detect replicaset name\"", claims.sub);
                return Err(RestError::UnauthorizedClient);
            }
        };

        let mut clusters: Vec<String> = Vec::new();
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        for scope in claims.scp {
            log::debug!("\"sub={}, Extracting scope {}", claims.sub, scope);

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
                            claims.sub
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
                            claims.sub,
                            action,
                            value
                        );
                        v.push(action);
                    }
                    None => {
                        log::debug!("\"sub={}, Adding role {} for {}\"", claims.sub, action, value);
                        let mut vec = Vec::new();
                        vec.push(action);
                        map.insert(value, vec);
                    }
                }
            } else if kind == "cluster" && action == "allow" {
                if !clusters.contains(&value) {
                    log::debug!(
                        "\"sub={}, Adding cluster access for replicaset {}\"",
                        claims.sub,
                        value
                    );
                    clusters.push(value)
                }
            }
        }

        // Ensure that client has access to current cluster
        let mut intersection = Vec::new();
        for cluster in clusters {
            if replicaset.contains(&cluster) {
                log::debug!("\"sub={}, Found replicaset intersection for {}\"", claims.sub, cluster);
                intersection.push(cluster);
            }
        };

        if intersection.len() == 0 {
            log::warn!(
                "\"sub={}, Did not find authorized replicaset\"",
                claims.sub,
                );
                return Err(RestError::UnauthorizedClient);
        };

        log::debug!("\"sub={}, Scope map: {:?}\"", claims.sub, map);
        Ok(AuthorizeScope {
            noauth: false,
            sub: claims.sub,
            jti: claims.jti,
            exp: Utc.timestamp(claims.exp, 0),
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
