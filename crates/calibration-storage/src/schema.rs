use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Storage {
    pub conn: Connection,
}

impl Storage {
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                created_at  INTEGER NOT NULL,
                updated_at  INTEGER NOT NULL,
                ended_at    INTEGER,
                state       TEXT NOT NULL,
                config_json TEXT NOT NULL,
                target_space TEXT NOT NULL,
                tier        TEXT,
                patch_count INTEGER,
                error_json  TEXT
            );

            CREATE TABLE IF NOT EXISTS patches (
                session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                patch_index INTEGER NOT NULL,
                patch_type  TEXT NOT NULL,
                target_rgb  TEXT NOT NULL,
                PRIMARY KEY (session_id, patch_index)
            );

            CREATE TABLE IF NOT EXISTS readings (
                session_id    TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                patch_index   INTEGER NOT NULL,
                reading_index INTEGER NOT NULL,
                raw_xyz       TEXT NOT NULL,
                measurement_type TEXT NOT NULL,
                measured_at   INTEGER NOT NULL,
                PRIMARY KEY (session_id, patch_index, reading_index, measurement_type)
            );

            CREATE TABLE IF NOT EXISTS computed_results (
                session_id   TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
                gamma        REAL,
                max_de       REAL,
                avg_de       REAL,
                lut_1d_json  TEXT,
                lut_3d_size  INTEGER,
                lut_3d_json  TEXT,
                white_balance TEXT,
                computed_at  INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS meter_initializations (
                meter_serial TEXT PRIMARY KEY,
                meter_model  TEXT NOT NULL,
                last_init_at TEXT NOT NULL,
                expires_at   TEXT NOT NULL
            );
            "#
        )?;

        crate::migration::run_migrations(&self.conn)?;

        Ok(())
    }
}
