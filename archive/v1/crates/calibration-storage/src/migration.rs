use rusqlite::{Connection, Result};

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2"
    )?;
    let exists: Option<i32> = stmt.query_row([table, column], |row| row.get(0)).ok();
    Ok(exists.is_some())
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current_version < 2 {
        if !column_exists(conn, "sessions", "ended_at")? {
            conn.execute("ALTER TABLE sessions ADD COLUMN ended_at INTEGER", [])?;
        }
        if !column_exists(conn, "sessions", "tier")? {
            conn.execute("ALTER TABLE sessions ADD COLUMN tier TEXT", [])?;
        }
        if !column_exists(conn, "sessions", "patch_count")? {
            conn.execute("ALTER TABLE sessions ADD COLUMN patch_count INTEGER", [])?;
        }
        if !column_exists(conn, "computed_results", "lut_3d_size")? {
            conn.execute("ALTER TABLE computed_results ADD COLUMN lut_3d_size INTEGER", [])?;
        }
        if !column_exists(conn, "computed_results", "lut_3d_json")? {
            conn.execute("ALTER TABLE computed_results ADD COLUMN lut_3d_json TEXT", [])?;
        }
        conn.execute("PRAGMA user_version = 2", [])?;
    }

    Ok(())
}
