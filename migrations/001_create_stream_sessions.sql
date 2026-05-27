CREATE TABLE stream_sessions (
    id UUID PRIMARY KEY,
    token_id TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    config_snapshot JSONB NOT NULL
);