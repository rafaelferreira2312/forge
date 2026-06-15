-- Guarda leads gerados pela landing publica do Forge.
CREATE TABLE IF NOT EXISTS lead_dna (
    id         TEXT PRIMARY KEY,
    dna        TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    email      TEXT NOT NULL,
    whatsapp   TEXT NOT NULL,
    source     TEXT NOT NULL DEFAULT 'landing',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
