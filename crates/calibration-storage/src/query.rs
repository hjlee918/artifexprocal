use calibration_core::state::SessionConfig;
use color_science::types::XYZ;
use rusqlite::{Connection, Result};
use serde_json;
use std::collections::HashMap;

pub struct SessionFilter {
    pub target_space: Option<String>,
    pub state: Option<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub search: Option<String>,
}

impl Default for SessionFilter {
    fn default() -> Self {
        Self {
            target_space: None,
            state: None,
            date_from: None,
            date_to: None,
            search: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub ended_at: Option<i64>,
    pub state: String,
    pub target_space: String,
    pub tier: Option<String>,
    pub patch_count: usize,
    pub gamma: Option<f64>,
    pub max_de: Option<f64>,
    pub avg_de: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ComputedResults {
    pub gamma: Option<f64>,
    pub max_de: Option<f64>,
    pub avg_de: Option<f64>,
    pub white_balance: Option<String>,
    pub lut_1d_size: Option<usize>,
    pub lut_3d_size: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct PatchReading {
    pub patch_index: usize,
    pub target_rgb: (f64, f64, f64),
    pub measured_xyz: XYZ,
    pub reading_index: usize,
    pub measurement_type: String,
}

#[derive(Debug, Clone)]
pub struct SessionDetail {
    pub summary: SessionSummary,
    pub config: SessionConfig,
    pub readings: Vec<PatchReading>,
    pub results: Option<ComputedResults>,
}

pub struct SessionQuery<'a> {
    conn: &'a Connection,
}

impl<'a> SessionQuery<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn list(
        &self,
        filter: &SessionFilter,
        page: usize,
        per_page: usize,
    ) -> Result<(Vec<SessionSummary>, usize)> {
        let mut where_clauses: Vec<String> = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref ts) = filter.target_space {
            where_clauses.push("s.target_space = ?".to_string());
            params_vec.push(Box::new(ts.clone()));
        }
        if let Some(ref st) = filter.state {
            where_clauses.push("s.state = ?".to_string());
            params_vec.push(Box::new(st.clone()));
        }
        if let Some(from) = filter.date_from {
            where_clauses.push("s.created_at >= ?".to_string());
            params_vec.push(Box::new(from));
        }
        if let Some(to) = filter.date_to {
            where_clauses.push("s.created_at <= ?".to_string());
            params_vec.push(Box::new(to));
        }
        if let Some(ref search) = filter.search {
            where_clauses.push("s.name LIKE ?".to_string());
            params_vec.push(Box::new(format!("%{}%", search)));
        }

        let where_sql = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!(
            "SELECT COUNT(*) FROM sessions s {}",
            where_sql
        );
        let total: usize = {
            let mut stmt = self.conn.prepare(&count_sql)?;
            let params_ref: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
            stmt.query_row(&*params_ref, |row| row.get(0))?
        };

        let select_sql = format!(
            "SELECT s.id, s.name, s.created_at, s.ended_at, s.state, s.target_space, s.tier,
                    s.patch_count, cr.gamma, cr.max_de, cr.avg_de
             FROM sessions s
             LEFT JOIN computed_results cr ON s.id = cr.session_id
             {}
             ORDER BY s.created_at DESC
             LIMIT ? OFFSET ?",
            where_sql
        );

        let offset = page * per_page;
        let mut stmt = self.conn.prepare(&select_sql)?;
        let mut params_ref: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        params_ref.push(&per_page);
        params_ref.push(&offset);

        let rows = stmt.query_map(&*params_ref, |row| {
            Ok(SessionSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                ended_at: row.get(3)?,
                state: row.get(4)?,
                target_space: row.get(5)?,
                tier: row.get(6)?,
                patch_count: row.get::<_, i64>(7)? as usize,
                gamma: row.get(8)?,
                max_de: row.get(9)?,
                avg_de: row.get(10)?,
            })
        })?;

        let items: Vec<SessionSummary> = rows.collect::<Result<Vec<_>>>()?;
        Ok((items, total))
    }

    pub fn get_detail(&self, session_id: &str) -> Result<Option<SessionDetail>> {
        let session_row = self.conn.query_row(
            "SELECT s.id, s.name, s.created_at, s.ended_at, s.state, s.target_space, s.tier,
                    s.patch_count, s.config_json, cr.gamma, cr.max_de, cr.avg_de, cr.white_balance,
                    cr.lut_1d_json, cr.lut_3d_size, cr.lut_3d_json
             FROM sessions s
             LEFT JOIN computed_results cr ON s.id = cr.session_id
             WHERE s.id = ?1",
            [session_id],
            |row| {
                let config_json: String = row.get(8)?;
                let config: SessionConfig = serde_json::from_str(&config_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;

                let lut_1d_json: Option<String> = row.get(13)?;
                let lut_1d_size = lut_1d_json.as_ref().map(|j| j.matches(',').count() + 1);

                Ok((
                    SessionSummary {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                        ended_at: row.get(3)?,
                        state: row.get(4)?,
                        target_space: row.get(5)?,
                        tier: row.get(6)?,
                        patch_count: row.get::<_, i64>(7)? as usize,
                        gamma: row.get(9)?,
                        max_de: row.get(10)?,
                        avg_de: row.get(11)?,
                    },
                    config,
                    ComputedResults {
                        gamma: row.get(9)?,
                        max_de: row.get(10)?,
                        avg_de: row.get(11)?,
                        white_balance: row.get(12)?,
                        lut_1d_size,
                        lut_3d_size: row.get(14)?,
                    },
                ))
            },
        );

        let (summary, config, results) = match session_row {
            Ok(data) => data,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e),
        };

        let mut stmt = self.conn.prepare(
            "SELECT patch_index, reading_index, raw_xyz, measurement_type
             FROM readings
             WHERE session_id = ?1
             ORDER BY patch_index, reading_index"
        )?;

        let reading_rows = stmt.query_map([session_id], |row| {
            let raw_json: String = row.get(2)?;
            let xyz: XYZ = serde_json::from_str(&raw_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?;

            Ok(PatchReading {
                patch_index: row.get::<_, i64>(0)? as usize,
                reading_index: row.get::<_, i64>(1)? as usize,
                target_rgb: (0.0, 0.0, 0.0),
                measured_xyz: xyz,
                measurement_type: row.get(3)?,
            })
        })?;

        let mut readings: Vec<PatchReading> = reading_rows.collect::<Result<Vec<_>>>()?;

        let mut patch_stmt = self.conn.prepare(
            "SELECT patch_index, target_rgb FROM patches WHERE session_id = ?1"
        )?;
        let patch_rows = patch_stmt.query_map([session_id], |row| {
            let rgb_json: String = row.get(1)?;
            let rgb: (f64, f64, f64) = serde_json::from_str(&rgb_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?;
            Ok((row.get::<_, i64>(0)? as usize, rgb))
        })?;

        let patch_map: HashMap<usize, (f64, f64, f64)> =
            patch_rows.collect::<Result<HashMap<_, _>>>()?;

        for reading in &mut readings {
            if let Some(rgb) = patch_map.get(&reading.patch_index) {
                reading.target_rgb = *rgb;
            }
        }

        Ok(Some(SessionDetail {
            summary,
            config,
            readings,
            results: Some(results),
        }))
    }
}
