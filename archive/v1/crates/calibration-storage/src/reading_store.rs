use color_science::types::XYZ;
use rusqlite::{Connection, Result, params};
use serde_json;

pub struct ReadingStore<'a> {
    conn: &'a Connection,
}

impl<'a> ReadingStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn save(
        &self,
        session_id: &str,
        patch_index: usize,
        reading_index: usize,
        xyz: &XYZ,
        measurement_type: &str,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let xyz_json = serde_json::to_string(xyz)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        self.conn.execute(
            "INSERT INTO readings (session_id, patch_index, reading_index, raw_xyz, measurement_type, measured_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(session_id, patch_index, reading_index, measurement_type) DO UPDATE SET
             raw_xyz = excluded.raw_xyz, measured_at = excluded.measured_at",
            params![
                session_id,
                patch_index as i64,
                reading_index as i64,
                xyz_json,
                measurement_type,
                now,
            ],
        )?;

        Ok(())
    }

    pub fn load_for_patch(
        &self,
        session_id: &str,
        patch_index: usize,
        measurement_type: &str,
    ) -> Result<Vec<XYZ>> {
        let mut stmt = self.conn.prepare(
            "SELECT raw_xyz FROM readings
             WHERE session_id = ?1 AND patch_index = ?2 AND measurement_type = ?3
             ORDER BY reading_index"
        )?;

        let rows = stmt.query_map(
            params![session_id, patch_index as i64, measurement_type],
            |row| {
                let json: String = row.get(0)?;
                let xyz: XYZ = serde_json::from_str(&json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;
                Ok(xyz)
            },
        )?;

        rows.collect::<Result<Vec<_>, _>>()
    }
}
