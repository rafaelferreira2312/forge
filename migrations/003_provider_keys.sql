CREATE TABLE IF NOT EXISTS provider_keys (
    provider   TEXT PRIMARY KEY,
    api_key    TEXT NOT NULL,
    validated  BOOLEAN DEFAULT FALSE,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS chat_history (
    id           TEXT PRIMARY KEY,
    input        TEXT NOT NULL,
    prompt_used  TEXT NOT NULL,
    response     TEXT NOT NULL,
    provider     TEXT NOT NULL,
    model        TEXT NOT NULL,
    tokens_used  INTEGER,
    rating       INTEGER,
    created_at   DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS learning_signals (
    id         TEXT PRIMARY KEY,
    input      TEXT NOT NULL,
    rating     INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
