use color_science::types::{RGB, XYZ};
use rusqlite::{Connection, Result, params};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StoredProfilingSession {
    pub id: String,
    pub name: String,
    pub state: String,
    pub field_meter_id: String,
    pub reference_meter_id: String,
    pub display_id: Option<String>,
    pub patch_count: Option<usize>,
    pub matrix: Option<[[f64; 3]; 3]>,
    pub accuracy: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ProfilingReading {
    pub patch_index: usize,
    pub target_rgb: RGB,
    pub reference_xyz: XYZ,
    pub meter_xyz: XYZ,
    pub delta_e: f64,
}

pub struct ProfilingSessionStore<'a> {
    conn: &'a Connection,
}

impl<'a> ProfilingSessionStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(
        &self,
        id: &str,
        name: &str,
        field_meter_id: &str,
        reference_meter_id: &str,
        display_id: Option<&str>,
    ) -> Result<String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        self.conn.execute(
            "INSERT INTO profiling_sessions (id, name, created_at, state, field_meter_id, reference_meter_id, display_id, patch_count, matrix_json, accuracy, error_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id,
                name,
                now,
                "running",
                field_meter_id,
                reference_meter_id,
                display_id,
                Option::<i64>::None,
                Option::<&str>::None,
                Option::<f64>::None,
                Option::<&str>::None,
            ],
        )?;

        Ok(id.to_string())
    }

    pub fn update_state(&self, id: &str, state: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let is_terminal = matches!(state, "finished" | "error" | "aborted");

        if is_terminal {
            self.conn.execute(
                "UPDATE profiling_sessions SET state = ?1, ended_at = ?2 WHERE id = ?3",
                params![state, now, id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE profiling_sessions SET state = ?1 WHERE id = ?2",
                params![state, id],
            )?;
        }
        Ok(())
    }

    pub fn save_result(
        &self,
        id: &str,
        matrix: &[[f64; 3]; 3],
        accuracy: f64,
        patch_count: usize,
    ) -> Result<()> {
        let matrix_json = serde_json::to_string(matrix)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        self.conn.execute(
            "UPDATE profiling_sessions SET matrix_json = ?1, accuracy = ?2, patch_count = ?3 WHERE id = ?4",
            params![matrix_json, accuracy, patch_count as i64, id],
        )?;
        Ok(())
    }

    pub fn save_reading(
        &self,
        session_id: &str,
        patch_index: usize,
        target_rgb: &RGB,
        reference_xyz: &XYZ,
        meter_xyz: &XYZ,
        delta_e: f64,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let rgb_json = serde_json::to_string(target_rgb)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let ref_json = serde_json::to_string(reference_xyz)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let meter_json = serde_json::to_string(meter_xyz)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        self.conn.execute(
            "INSERT INTO profiling_readings (session_id, patch_index, target_rgb, reference_xyz, meter_xyz, delta_e, measured_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(session_id, patch_index) DO UPDATE SET
             target_rgb = excluded.target_rgb, reference_xyz = excluded.reference_xyz,
             meter_xyz = excluded.meter_xyz, delta_e = excluded.delta_e, measured_at = excluded.measured_at",
            params![
                session_id,
                patch_index as i64,
                rgb_json,
                ref_json,
                meter_json,
                delta_e,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<StoredProfilingSession> {
        self.conn.query_row(
            "SELECT id, name, state, field_meter_id, reference_meter_id, display_id, patch_count, matrix_json, accuracy FROM profiling_sessions WHERE id = ?1",
            [id],
            |row| {
                let matrix_json: Option<String> = row.get(7)?;
                let matrix = matrix_json.and_then(|s| serde_json::from_str::<[[f64; 3]; 3]>(&s).ok());

                Ok(StoredProfilingSession {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    state: row.get(2)?,
                    field_meter_id: row.get(3)?,
                    reference_meter_id: row.get(4)?,
                    display_id: row.get(5)?,
                    patch_count: row.get::<_, Option<i64>>(6)?.map(|v| v as usize),
                    matrix,
                    accuracy: row.get(8)?,
                })
            },
        )
    }

    pub fn load_readings(&self,
        session_id: &str,
    ) -> Result<Vec<ProfilingReading>> {
        let mut stmt = self.conn.prepare(
            "SELECT patch_index, target_rgb, reference_xyz, meter_xyz, delta_e
             FROM profiling_readings WHERE session_id = ?1 ORDER BY patch_index"
        )?;

        let rows = stmt.query_map([session_id], |row| {
            let rgb_json: String = row.get(1)?;
            let rgb: RGB = serde_json::from_str(&rgb_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    1, rusqlite::types::Type::Text, Box::new(e),
                ))?;
            let ref_json: String = row.get(2)?;
            let ref_xyz: XYZ = serde_json::from_str(&ref_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    2, rusqlite::types::Type::Text, Box::new(e),
                ))?;
            let meter_json: String = row.get(3)?;
            let meter_xyz: XYZ = serde_json::from_str(&meter_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    3, rusqlite::types::Type::Text, Box::new(e),
                ))?;

            Ok(ProfilingReading {
                patch_index: row.get::<_, i64>(0)? as usize,
                target_rgb: rgb,
                reference_xyz: ref_xyz,
                meter_xyz: meter_xyz,
                delta_e: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Storage;

    #[test]
    fn test_create_and_get_session() {
        let storage = Storage::new_in_memory().unwrap();
        let store = ProfilingSessionStore::new(&storage.conn);

        let id = store.create("test-id", "Test Profile", "i1-display-pro", "i1-pro-2", Some("lg-oled")).unwrap();
        let session = store.get(&id).unwrap();

        assert_eq!(session.name, "Test Profile");
        assert_eq!(session.field_meter_id, "i1-display-pro");
        assert_eq!(session.reference_meter_id, "i1-pro-2");
        assert_eq!(session.display_id, Some("lg-oled".to_string()));
        assert_eq!(session.state, "running");
        assert!(session.matrix.is_none());
    }

    #[test]
    fn test_save_result_and_readings() {
        let storage = Storage::new_in_memory().unwrap();
        let store = ProfilingSessionStore::new(&storage.conn);

        let id = store.create("test-id", "Test", "meter1", "meter2", None).unwrap();

        store.save_reading(
            &id, 0,
            &RGB { r: 1.0, g: 0.0, b: 0.0 },
            &XYZ { x: 10.0, y: 5.0, z: 2.0 },
            &XYZ { x: 11.0, y: 5.5, z: 2.1 },
            0.35,
        ).unwrap();

        let matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        store.save_result(&id, &matrix, 0.12, 20,
        ).unwrap();

        let session = store.get(&id).unwrap();
        assert!(session.matrix.is_some());
        assert!((session.accuracy.unwrap() - 0.12).abs() < 1e-6);
        assert_eq!(session.patch_count, Some(20));

        let readings = store.load_readings(&id).unwrap();
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].patch_index, 0);
        assert!((readings[0].delta_e - 0.35).abs() < 1e-6);
    }
}