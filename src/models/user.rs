use crate::schema::{users, grants, mobile_sessions};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub google_sub: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub google_sub: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = grants)]
pub struct Grant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub scope: String,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = grants)]
pub struct NewGrant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = mobile_sessions)]
pub struct MobileSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_token: String,
    pub platform: String,
    pub app_version: String,
    pub last_heartbeat: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = mobile_sessions)]
pub struct NewMobileSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_token: String,
    pub platform: String,
    pub app_version: String,
}

impl User {
    pub fn new(google_sub: String, email: String, name: String, picture: Option<String>) -> NewUser {
        NewUser {
            id: Uuid::new_v4(),
            google_sub,
            email,
            name,
            picture,
        }
    }
}
