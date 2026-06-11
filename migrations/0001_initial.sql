-- Placeholder for future persistence of engineered prompts.
-- The initial version keeps history and statistics in memory.
CREATE TABLE IF NOT EXISTS prompt_history (
    id INTEGER PRIMARY KEY,
    created_at_unix INTEGER NOT NULL,
    input TEXT NOT NULL,
    provider TEXT NOT NULL,
    intent TEXT NOT NULL,
    domain TEXT NOT NULL,
    complexity TEXT NOT NULL,
    technique TEXT NOT NULL,
    prompt TEXT NOT NULL
);