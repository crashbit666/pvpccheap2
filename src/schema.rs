// @generated automatically by Diesel CLI.

diesel::table! {
    automation_logs (id) {
        id -> Uuid,
        user_id -> Uuid,
        device_id -> Nullable<Uuid>,
        rule_id -> Nullable<Uuid>,
        action -> Varchar,
        details_json -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    commands (id) {
        id -> Uuid,
        user_id -> Uuid,
        device_id -> Uuid,
        command_type -> Varchar,
        payload_json -> Jsonb,
        status -> Varchar,
        retry_count -> Int4,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
        executed_at -> Nullable<Timestamptz>,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    day_prices (id) {
        id -> Uuid,
        date -> Date,
        timezone -> Varchar,
        prices_json -> Jsonb,
        source -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    device_states (id) {
        id -> Uuid,
        device_id -> Uuid,
        state_json -> Jsonb,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    devices (id) {
        id -> Uuid,
        user_id -> Uuid,
        structure_id -> Nullable<Uuid>,
        google_device_id -> Varchar,
        name -> Varchar,
        device_type -> Varchar,
        room -> Nullable<Varchar>,
        capabilities_json -> Jsonb,
        last_seen_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    grants (id) {
        id -> Uuid,
        user_id -> Uuid,
        platform -> Varchar,
        scope -> Varchar,
        granted_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    mobile_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        device_token -> Varchar,
        platform -> Varchar,
        app_version -> Varchar,
        last_heartbeat -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    rules (id) {
        id -> Uuid,
        user_id -> Uuid,
        device_id -> Uuid,
        rule_type -> Varchar,
        params_json -> Jsonb,
        timezone -> Varchar,
        priority -> Int4,
        enabled -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    schedules (id) {
        id -> Uuid,
        user_id -> Uuid,
        device_id -> Uuid,
        rule_id -> Uuid,
        date -> Date,
        slots_json -> Jsonb,
        total_cost -> Numeric,
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    structures (id) {
        id -> Uuid,
        user_id -> Uuid,
        google_structure_id -> Varchar,
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        google_sub -> Varchar,
        email -> Varchar,
        name -> Varchar,
        picture -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(automation_logs -> devices (device_id));
diesel::joinable!(automation_logs -> rules (rule_id));
diesel::joinable!(automation_logs -> users (user_id));
diesel::joinable!(commands -> devices (device_id));
diesel::joinable!(commands -> users (user_id));
diesel::joinable!(device_states -> devices (device_id));
diesel::joinable!(devices -> structures (structure_id));
diesel::joinable!(devices -> users (user_id));
diesel::joinable!(grants -> users (user_id));
diesel::joinable!(mobile_sessions -> users (user_id));
diesel::joinable!(rules -> devices (device_id));
diesel::joinable!(rules -> users (user_id));
diesel::joinable!(schedules -> devices (device_id));
diesel::joinable!(schedules -> rules (rule_id));
diesel::joinable!(schedules -> users (user_id));
diesel::joinable!(structures -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    automation_logs,
    commands,
    day_prices,
    device_states,
    devices,
    grants,
    mobile_sessions,
    rules,
    schedules,
    structures,
    users,
);
