-- Disk Helper v1 initial schema (see docs/current/modules/disk-helper/详细设计_v1.md)

CREATE TABLE IF NOT EXISTS scan_run (
    id TEXT PRIMARY KEY NOT NULL,
    scan_type TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    scanned_files INTEGER NOT NULL DEFAULT 0,
    skipped_files INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS file_entry (
    path TEXT PRIMARY KEY NOT NULL,
    parent_path TEXT NOT NULL,
    name TEXT NOT NULL,
    is_dir INTEGER NOT NULL,
    size_bytes INTEGER NOT NULL,
    folder_size INTEGER NOT NULL,
    modified_at TEXT,
    extension TEXT,
    coverage TEXT NOT NULL,
    scan_run_id TEXT,
    FOREIGN KEY (scan_run_id) REFERENCES scan_run (id)
);

CREATE INDEX IF NOT EXISTS idx_file_entry_parent ON file_entry (parent_path);
CREATE INDEX IF NOT EXISTS idx_file_entry_size ON file_entry (size_bytes DESC);
CREATE INDEX IF NOT EXISTS idx_file_entry_folder_size ON file_entry (folder_size DESC);
CREATE INDEX IF NOT EXISTS idx_file_entry_name ON file_entry (name COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS scan_skip (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_run_id TEXT NOT NULL,
    path TEXT NOT NULL,
    reason TEXT NOT NULL,
    FOREIGN KEY (scan_run_id) REFERENCES scan_run (id)
);

CREATE TABLE IF NOT EXISTS quarantine_item (
    id TEXT PRIMARY KEY NOT NULL,
    original_path TEXT NOT NULL,
    quarantine_path TEXT NOT NULL UNIQUE,
    size_bytes INTEGER NOT NULL,
    moved_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    risk TEXT NOT NULL,
    rule_id TEXT
);

CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    occurred_at TEXT NOT NULL,
    event_type TEXT NOT NULL,
    summary TEXT NOT NULL,
    result TEXT NOT NULL,
    detail_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_audit_occurred ON audit_log (occurred_at DESC);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
