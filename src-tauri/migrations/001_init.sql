CREATE TABLE IF NOT EXISTS verification_cache (
    claim_hash      TEXT PRIMARY KEY,
    claim_text      TEXT NOT NULL,
    verification    TEXT NOT NULL,
    created_at_ms   INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cache_created
    ON verification_cache(created_at_ms DESC);

CREATE TABLE IF NOT EXISTS analysis_history (
    id              TEXT PRIMARY KEY,
    created_at_ms   INTEGER NOT NULL,
    input           TEXT NOT NULL,
    analysis_json   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_history_created
    ON analysis_history(created_at_ms DESC);
