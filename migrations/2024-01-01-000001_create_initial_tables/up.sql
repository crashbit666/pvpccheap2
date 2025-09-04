-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    google_sub VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    picture VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create grants table
CREATE TABLE grants (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform VARCHAR NOT NULL, -- 'google_home', 'alexa', etc.
    scope VARCHAR NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- Create structures table
CREATE TABLE structures (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    google_structure_id VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, google_structure_id)
);

-- Create devices table
CREATE TABLE devices (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    structure_id UUID REFERENCES structures(id) ON DELETE SET NULL,
    google_device_id VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    device_type VARCHAR NOT NULL,
    room VARCHAR,
    capabilities_json JSONB NOT NULL DEFAULT '{}',
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, google_device_id)
);

-- Create device_states table
CREATE TABLE device_states (
    id UUID PRIMARY KEY,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    state_json JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(device_id)
);

-- Create commands table
CREATE TABLE commands (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    command_type VARCHAR NOT NULL,
    payload_json JSONB NOT NULL DEFAULT '{}',
    status VARCHAR NOT NULL DEFAULT 'queued', -- 'queued', 'sent', 'acked', 'failed'
    retry_count INT NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    executed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create rules table
CREATE TABLE rules (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    rule_type VARCHAR NOT NULL, -- 'MIN_HOURS_CHEAPEST', 'X_HOURS_WITHIN_WINDOWS'
    params_json JSONB NOT NULL DEFAULT '{}',
    timezone VARCHAR NOT NULL DEFAULT 'Europe/Madrid',
    priority INT NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create day_prices table
CREATE TABLE day_prices (
    id UUID PRIMARY KEY,
    date DATE NOT NULL,
    timezone VARCHAR NOT NULL,
    prices_json JSONB NOT NULL, -- Array de 24 preus
    source VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(date, timezone)
);

-- Create schedules table
CREATE TABLE schedules (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    rule_id UUID NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    slots_json JSONB NOT NULL, -- Array d'intervals [{start: "00:00", end: "06:00"}]
    total_cost NUMERIC(10,4) NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'pending', -- 'pending', 'active', 'completed', 'failed'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(device_id, date)
);

-- Create automation_logs table
CREATE TABLE automation_logs (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id UUID REFERENCES devices(id) ON DELETE SET NULL,
    rule_id UUID REFERENCES rules(id) ON DELETE SET NULL,
    action VARCHAR NOT NULL,
    details_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create mobile_sessions table
CREATE TABLE mobile_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_token VARCHAR NOT NULL, -- FCM token
    platform VARCHAR NOT NULL, -- 'android', 'ios'
    app_version VARCHAR NOT NULL,
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, device_token)
);

-- Create indexes for performance
CREATE INDEX idx_devices_user_id ON devices(user_id);
CREATE INDEX idx_devices_structure_id ON devices(structure_id);
CREATE INDEX idx_commands_user_id ON commands(user_id);
CREATE INDEX idx_commands_device_id ON commands(device_id);
CREATE INDEX idx_commands_status ON commands(status);
CREATE INDEX idx_rules_user_id ON rules(user_id);
CREATE INDEX idx_rules_device_id ON rules(device_id);
CREATE INDEX idx_rules_enabled ON rules(enabled);
CREATE INDEX idx_schedules_user_id ON schedules(user_id);
CREATE INDEX idx_schedules_device_id ON schedules(device_id);
CREATE INDEX idx_schedules_date ON schedules(date);
CREATE INDEX idx_schedules_status ON schedules(status);
CREATE INDEX idx_day_prices_date ON day_prices(date);
CREATE INDEX idx_automation_logs_user_id ON automation_logs(user_id);
CREATE INDEX idx_automation_logs_created_at ON automation_logs(created_at);
CREATE INDEX idx_mobile_sessions_user_id ON mobile_sessions(user_id);
CREATE INDEX idx_mobile_sessions_last_heartbeat ON mobile_sessions(last_heartbeat);
