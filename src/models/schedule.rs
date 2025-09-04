use crate::schema::{schedules, day_prices};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schedules)]
pub struct Schedule {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub rule_id: Uuid,
    pub date: NaiveDate,
    pub slots_json: JsonValue, // Array de TimeSlot
    pub total_cost: Decimal,
    pub status: String, // "pending", "active", "completed", "failed"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = schedules)]
pub struct NewSchedule {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub rule_id: Uuid,
    pub date: NaiveDate,
    pub slots_json: JsonValue,
    pub total_cost: Decimal,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = day_prices)]
pub struct DayPrice {
    pub id: Uuid,
    pub date: NaiveDate,
    pub timezone: String,
    pub prices_json: JsonValue, // Array de 24 preus (o 23/25 en canvi horari)
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = day_prices)]
pub struct NewDayPrice {
    pub id: Uuid,
    pub date: NaiveDate,
    pub timezone: String,
    pub prices_json: JsonValue,
    pub source: String,
}

// Estructures per als slots de temps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start: String,  // Format "HH:MM"
    pub end: String,    // Format "HH:MM"
    pub action: String, // "on" o "off"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyPrice {
    pub hour: u8,
    pub price: Decimal,
}

// DTO per a la resposta de l'horari calculat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResponse {
    pub device_id: Uuid,
    pub date: NaiveDate,
    pub slots: Vec<TimeSlot>,
    pub total_cost: Decimal,
    pub total_hours: f32,
    pub savings_percentage: Option<f32>, // Comparació amb no optimitzar
}

// DTO per a sol·licitar previsualització
#[derive(Debug, Clone, Deserialize)]
pub struct PreviewScheduleRequest {
    pub device_id: Uuid,
    pub rule_id: Option<Uuid>,
    pub date: NaiveDate,
    pub rule_params: Option<JsonValue>, // Per previsualitzar sense crear la regla
}

impl Schedule {
    pub fn get_slots(&self) -> Result<Vec<TimeSlot>, serde_json::Error> {
        serde_json::from_value(self.slots_json.clone())
    }
    
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
    
    pub fn is_pending(&self) -> bool {
        self.status == "pending"
    }
}
