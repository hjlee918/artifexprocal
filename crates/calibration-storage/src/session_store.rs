use calibration_core::state::{SessionConfig, TargetSpace};
use rusqlite::{Connection, Result, params};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StoredSession {
    pub id: String,
    pub name: String,
    pub state: String,
    pub config: SessionConfig,
    pub target_space: String,
    pub error_json: Option<String>,
}

pub struct SessionStore<'a> {
    conn: &'a Connection,
}

impl<'a> SessionStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, config: &SessionConfig) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let config_json = serde_json::to_string(config)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let target_space = match &config.target_space {
            TargetSpace::Bt709 => "BT.709",
            TargetSpace::Bt2020 => "BT.2020",
            TargetSpace::DciP3 => "DCI-P3",
            TargetSpace::Custom { .. } => "Custom",
        };

        self.conn.execute(
            "INSERT INTO sessions (id, name, created_at, updated_at, state, config_json, target_space, error_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &id,
                &config.name,
                now,
                now,
                "idle",
                &config_json,
                target_space,
                Option::<&str>::None,
            ],
        )?;

        Ok(id)
    }

    pub fn get(&self, id: &str) -> Result<StoredSession> {
        self.conn.query_row(
            "SELECT id, name, state, config_json, target_space, error_json FROM sessions WHERE id = ?1",
            [id],
            |row| {
                let config_json: String = row.get(3)?;
                let config: SessionConfig = serde_json::from_str(&config_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;

                Ok(StoredSession {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    state: row.get(2)?,
                    config,
                    target_space: row.get(4)?,
                    error_json: row.get(5)?,
                })
            },
        )
    }

    pub fn update_state(&self, id: &str, state: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        self.conn.execute(
            "UPDATE sessions SET state = ?1, updated_at = ?2 WHERE id = ?3",
            params![state, now, id],
        )?;
        Ok(())
    }
}
