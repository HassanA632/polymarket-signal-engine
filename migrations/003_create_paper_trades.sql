CREATE TABLE paper_trades (
    id BIGSERIAL PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES stream_sessions(id) ON DELETE CASCADE,
    token_id TEXT NOT NULL,
    side TEXT NOT NULL,
    entry_price DOUBLE PRECISION NOT NULL,
    exit_price DOUBLE PRECISION NOT NULL,
    stake DOUBLE PRECISION NOT NULL,
    pnl DOUBLE PRECISION NOT NULL,
    exit_reason TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_paper_trades_session_id ON paper_trades(session_id);
CREATE INDEX idx_paper_trades_token_id ON paper_trades(token_id);
CREATE INDEX idx_paper_trades_created_at ON paper_trades(created_at);