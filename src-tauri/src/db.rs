use rusqlite::{params, Connection, Result};
use crate::state::AppState;

pub fn init(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS events (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            action    TEXT NOT NULL,
            reminder_type TEXT DEFAULT 'stretch',
            source    TEXT DEFAULT 'manual'
        );
        CREATE TABLE IF NOT EXISTS app_state (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

pub fn save_state(conn: &Connection, state: &AppState) -> Result<()> {
    let json = serde_json::to_string(state).map_err(|e| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
    })?;
    conn.execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES ('state', ?1)",
        params![json],
    )?;
    Ok(())
}

pub fn load_state(conn: &Connection) -> Option<AppState> {
    let result: rusqlite::Result<String> = conn.query_row(
        "SELECT value FROM app_state WHERE key = 'state'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(json) => serde_json::from_str(&json).ok(),
        Err(_) => None,
    }
}

pub fn log_event(conn: &Connection, action: &str, source: &str) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    conn.execute(
        "INSERT INTO events (timestamp, action, source) VALUES (?1, ?2, ?3)",
        params![ts, action, source],
    )?;
    Ok(())
}
