use crate::schema::{commands, automation_logs};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Queued,
    Sent,
    Acked,
    Failed,
}

impl ToString for CommandStatus {
    fn to_string(&self) -> String {
        match self {
            CommandStatus::Queued => "queued".to_string(),
            CommandStatus::Sent => "sent".to_string(),
            CommandStatus::Acked => "acked".to_string(),
            CommandStatus::Failed => "failed".to_string(),
        }
    }
}

impl From<String> for CommandStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "queued" => CommandStatus::Queued,
            "sent" => CommandStatus::Sent,
            "acked" => CommandStatus::Acked,
            "failed" => CommandStatus::Failed,
            _ => CommandStatus::Queued,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = commands)]
pub struct Command {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub command_type: String,
    pub payload_json: JsonValue,
    pub status: String,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = commands)]
pub struct NewCommand {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub command_type: String,
    pub payload_json: JsonValue,
    pub status: String,
    pub retry_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = automation_logs)]
pub struct AutomationLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Option<Uuid>,
    pub rule_id: Option<Uuid>,
    pub action: String,
    pub details_json: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = automation_logs)]
pub struct NewAutomationLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Option<Uuid>,
    pub rule_id: Option<Uuid>,
    pub action: String,
    pub details_json: Option<JsonValue>,
}

// DTOs per a l'API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateCommandRequest {
    pub device_id: Uuid,
    pub command_type: String, // "on_off", "brightness", "temperature", etc.
    pub payload: JsonValue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandResult {
    pub command_id: Uuid,
    pub success: bool,
    pub error_message: Option<String>,
    pub new_state: Option<JsonValue>,
    pub executed_at: DateTime<Utc>,
}

// Payloads específics per tipus de comanda
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnOffPayload {
    pub on: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrightnessPayload {
    pub brightness: u8, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperaturePayload {
    pub temperature: f32,
    pub unit: String, // "celsius" o "fahrenheit"
}

impl Command {
    pub fn get_status(&self) -> CommandStatus {
        CommandStatus::from(self.status.clone())
    }
    
    pub fn is_retriable(&self) -> bool {
        self.retry_count < 3 && self.status != "acked"
    }
    
    pub fn should_expire(&self) -> bool {
        // Expira després de 5 minuts sense ser executat
        if let CommandStatus::Queued = self.get_status() {
            let elapsed = Utc::now().signed_duration_since(self.created_at);
            elapsed.num_minutes() > 5
        } else {
            false
        }
    }
}
