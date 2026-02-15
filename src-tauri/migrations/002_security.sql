-- Enable foreign keys globally
PRAGMA foreign_keys = ON;

-- Create schema version tracking table
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Add CASCADE to audit_log foreign key
CREATE TABLE audit_log_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    escalation_id INTEGER NOT NULL REFERENCES escalations(id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO audit_log_new SELECT * FROM audit_log;
DROP TABLE audit_log;
ALTER TABLE audit_log_new RENAME TO audit_log;

CREATE INDEX IF NOT EXISTS idx_audit_log_escalation ON audit_log(escalation_id);

-- Remove Jira credential columns from api_config (now in keychain)
-- Keep email as it's needed to retrieve credentials from keychain
CREATE TABLE api_config_new (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    jira_email TEXT NOT NULL DEFAULT '',
    ollama_endpoint TEXT NOT NULL DEFAULT 'http://localhost:11434',
    ollama_model TEXT NOT NULL DEFAULT 'llama3',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Migrate existing config (if exists)
INSERT OR IGNORE INTO api_config_new (id, jira_email, ollama_endpoint, ollama_model, updated_at)
SELECT id,
       COALESCE(jira_email, ''),
       COALESCE(ollama_endpoint, 'http://localhost:11434'),
       COALESCE(ollama_model, 'llama3'),
       updated_at
FROM api_config;

DROP TABLE api_config;
ALTER TABLE api_config_new RENAME TO api_config;

-- Record migration
INSERT INTO schema_migrations (version) VALUES (2);
