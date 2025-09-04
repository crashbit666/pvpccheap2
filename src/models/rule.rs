use crate::schema::rules;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleType {
    MinHoursCheapest,
    XHoursWithinWindows,
}

impl ToString for RuleType {
    fn to_string(&self) -> String {
        match self {
            RuleType::MinHoursCheapest => "MIN_HOURS_CHEAPEST".to_string(),
            RuleType::XHoursWithinWindows => "X_HOURS_WITHIN_WINDOWS".to_string(),
        }
    }
}

impl From<String> for RuleType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "MIN_HOURS_CHEAPEST" => RuleType::MinHoursCheapest,
            "X_HOURS_WITHIN_WINDOWS" => RuleType::XHoursWithinWindows,
            _ => RuleType::MinHoursCheapest,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = rules)]
pub struct Rule {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub rule_type: String,
    pub params_json: JsonValue,
    pub timezone: String,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = rules)]
pub struct NewRule {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub rule_type: String,
    pub params_json: JsonValue,
    pub timezone: String,
    pub priority: i32,
    pub enabled: bool,
}

// Paràmetres específics per a cada tipus de regla
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinHoursCheapestParams {
    pub min_hours_per_day: u8,
    #[serde(default)]
    pub max_switches_per_day: Option<u8>,
    #[serde(default)]
    pub min_run_block: Option<u8>, // Mínim d'hores consecutives
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XHoursWithinWindowsParams {
    pub target_hours_per_day: u8,
    pub allowed_windows: Vec<TimeWindow>,
    #[serde(default)]
    pub max_switches_per_day: Option<u8>,
    #[serde(default)]
    pub min_run_block: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: String, // Format "HH:MM"
    pub end: String,   // Format "HH:MM"
}

// DTO per a crear/editar regles des de l'app
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRuleRequest {
    pub device_id: Uuid,
    pub rule_type: RuleType,
    pub params: JsonValue,
    pub timezone: String,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRuleRequest {
    pub params: Option<JsonValue>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

impl Rule {
    pub fn get_rule_type(&self) -> RuleType {
        RuleType::from(self.rule_type.clone())
    }
    
    pub fn is_active(&self) -> bool {
        self.enabled
    }
}
