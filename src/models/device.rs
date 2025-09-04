use crate::schema::{devices, device_states, structures};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = structures)]
pub struct Structure {
    pub id: Uuid,
    pub user_id: Uuid,
    pub google_structure_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = structures)]
pub struct NewStructure {
    pub id: Uuid,
    pub user_id: Uuid,
    pub google_structure_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = devices)]
pub struct Device {
    pub id: Uuid,
    pub user_id: Uuid,
    pub structure_id: Option<Uuid>,
    pub google_device_id: String,
    pub name: String,
    pub device_type: String,
    pub room: Option<String>,
    pub capabilities_json: JsonValue,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = devices)]
pub struct NewDevice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub structure_id: Option<Uuid>,
    pub google_device_id: String,
    pub name: String,
    pub device_type: String,
    pub room: Option<String>,
    pub capabilities_json: JsonValue,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = device_states)]
pub struct DeviceState {
    pub id: Uuid,
    pub device_id: Uuid,
    pub state_json: JsonValue,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = device_states)]
pub struct NewDeviceState {
    pub id: Uuid,
    pub device_id: Uuid,
    pub state_json: JsonValue,
    pub updated_at: DateTime<Utc>,
}

// DTOs per a la sincronització des de l'app mòbil
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceSyncRequest {
    pub devices: Vec<DeviceSync>,
    pub structures: Vec<StructureSync>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceSync {
    pub google_device_id: String,
    pub name: String,
    pub device_type: String,
    pub room: Option<String>,
    pub structure_id: Option<String>,
    pub capabilities: JsonValue,
    pub state: JsonValue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructureSync {
    pub google_structure_id: String,
    pub name: String,
}
