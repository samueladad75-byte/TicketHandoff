CREATE TABLE IF NOT EXISTS templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL,
    checklist_items TEXT NOT NULL,
    l2_team TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS escalations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticket_id TEXT NOT NULL,
    template_id INTEGER REFERENCES templates(id),
    problem_summary TEXT NOT NULL DEFAULT '',
    checklist TEXT NOT NULL DEFAULT '[]',
    current_status TEXT NOT NULL DEFAULT '',
    next_steps TEXT NOT NULL DEFAULT '',
    llm_summary TEXT,
    llm_confidence TEXT,
    markdown_output TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    posted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    escalation_id INTEGER NOT NULL REFERENCES escalations(id),
    action TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS api_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    jira_base_url TEXT NOT NULL,
    jira_email TEXT NOT NULL,
    jira_api_token TEXT NOT NULL,
    ollama_endpoint TEXT NOT NULL DEFAULT 'http://localhost:11434',
    ollama_model TEXT NOT NULL DEFAULT 'llama3',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_escalations_ticket_id ON escalations(ticket_id);
CREATE INDEX IF NOT EXISTS idx_escalations_created_at ON escalations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_escalation ON audit_log(escalation_id);
