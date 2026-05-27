CREATE TABLE signals (
    id BIGSERIAL PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES stream_sessions(id) ON DELETE CASCADE,
    signal_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_signals_session_id ON signals(session_id);
CREATE INDEX idx_signals_signal_type ON signals(signal_type);
CREATE INDEX idx_signals_created_at ON signals(created_at);