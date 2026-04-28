use crate::query::SessionDetail;
use std::io::Write;

pub struct SessionExporter;

impl SessionExporter {
    pub fn export_csv(detail: &SessionDetail, writer: &mut dyn Write) -> std::io::Result<()> {
        writeln!(
            writer,
            "patch_index,target_r,target_g,target_b,measured_x,measured_y,measured_z,reading_index,measurement_type"
        )?;

        for reading in &detail.readings {
            writeln!(
                writer,
                "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{}",
                reading.patch_index,
                reading.target_rgb.0,
                reading.target_rgb.1,
                reading.target_rgb.2,
                reading.measured_xyz.x,
                reading.measured_xyz.y,
                reading.measured_xyz.z,
                reading.reading_index,
                reading.measurement_type,
            )?;
        }

        Ok(())
    }

    pub fn export_json(detail: &SessionDetail, writer: &mut dyn Write) -> std::io::Result<()> {
        let mut readings_json = Vec::new();
        for reading in &detail.readings {
            readings_json.push(serde_json::json!({
                "patch_index": reading.patch_index,
                "target_rgb": [reading.target_rgb.0, reading.target_rgb.1, reading.target_rgb.2],
                "measured_xyz": {
                    "x": reading.measured_xyz.x,
                    "y": reading.measured_xyz.y,
                    "z": reading.measured_xyz.z,
                },
                "reading_index": reading.reading_index,
                "measurement_type": reading.measurement_type,
            }));
        }

        let output = serde_json::json!({
            "session_id": detail.summary.id,
            "name": detail.summary.name,
            "created_at": detail.summary.created_at,
            "ended_at": detail.summary.ended_at,
            "state": detail.summary.state,
            "target_space": detail.summary.target_space,
            "tier": detail.summary.tier,
            "patch_count": detail.summary.patch_count,
            "config": {
                "name": detail.config.name,
                "target_space": format!("{:?}", detail.config.target_space),
                "tone_curve": format!("{:?}", detail.config.tone_curve),
                "white_point": format!("{:?}", detail.config.white_point),
                "patch_count": detail.config.patch_count,
                "reads_per_patch": detail.config.reads_per_patch,
                "settle_time_ms": detail.config.settle_time_ms,
                "stability_threshold": detail.config.stability_threshold,
                "tier": format!("{:?}", detail.config.tier),
            },
            "results": detail.results.as_ref().map(|r| serde_json::json!({
                "gamma": r.gamma,
                "max_de": r.max_de,
                "avg_de": r.avg_de,
                "white_balance": r.white_balance,
                "lut_1d_size": r.lut_1d_size,
                "lut_3d_size": r.lut_3d_size,
            })),
            "readings": readings_json,
        });

        serde_json::to_writer_pretty(writer, &output)?;
        Ok(())
    }
}
