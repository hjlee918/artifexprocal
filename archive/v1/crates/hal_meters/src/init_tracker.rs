use chrono::{DateTime, Utc, Duration};
use rusqlite::Connection;

#[derive(Debug, thiserror::Error)]
pub enum InitTrackerError {
    #[error("Database error: {0}")]
    Database(String),
}

pub struct InitTracker;

impl InitTracker {
    pub fn record_init(
        conn: &Connection,
        serial: &str,
        model: &str,
    ) -> Result<(), InitTrackerError> {
        let now = Utc::now();
        let expires = now + Duration::hours(3);
        conn.execute(
            "INSERT INTO meter_initializations (meter_serial, meter_model, last_init_at, expires_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(meter_serial) DO UPDATE SET
               meter_model = excluded.meter_model,
               last_init_at = excluded.last_init_at,
               expires_at = excluded.expires_at",
            [
                serial,
                model,
                &now.to_rfc3339(),
                &expires.to_rfc3339(),
            ],
        )
        .map_err(|e| InitTrackerError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn time_until_next_init(
        conn: &Connection,
        serial: &str,
    ) -> Option<std::time::Duration> {
        let expires_str: String = conn
            .query_row(
                "SELECT expires_at FROM meter_initializations WHERE meter_serial = ?1",
                [serial],
                |row| row.get(0),
            )
            .ok()?;
        let expires: DateTime<Utc> =
            DateTime::parse_from_rfc3339(&expires_str).ok()?.into();
        let now = Utc::now();
        let diff = expires.signed_duration_since(now);
        let secs = diff.num_seconds().max(0) as u64;
        Some(std::time::Duration::from_secs(secs))
    }

    pub fn is_init_expired(conn: &Connection, serial: &str) -> bool {
        match Self::time_until_next_init(conn, serial) {
            Some(d) => d.as_secs() == 0,
            None => true,
        }
    }
}
