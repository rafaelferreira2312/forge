-- Adiciona expertise_level ao perfil adaptativo
CREATE TABLE IF NOT EXISTS adaptive_profiles (
    domain            TEXT PRIMARY KEY,
    interaction_count INTEGER NOT NULL DEFAULT 0,
    confidence        REAL NOT NULL DEFAULT 0.1,
    updated_at        DATETIME DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE adaptive_profiles
ADD COLUMN expertise_level TEXT NOT NULL DEFAULT 'SeniorDev';

-- Tabela de configuração global do usuário
CREATE TABLE IF NOT EXISTS user_config (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Valor padrão inicial
INSERT OR IGNORE INTO user_config (key, value)
VALUES ('expertise_level', 'SeniorDev');
