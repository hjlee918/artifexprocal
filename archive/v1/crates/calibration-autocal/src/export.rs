use hal::types::Lut3D;
use std::io::Write;

pub struct Lut3DExporter;

impl Lut3DExporter {
    /// Export to DaVinci Resolve / Photoshop `.cube` format.
    pub fn export_cube<W: Write>(lut: &Lut3D, writer: &mut W) -> std::io::Result<()> {
        writeln!(writer, "# ArtifexProCal 3D LUT")?;
        writeln!(writer, "TITLE \"ArtifexProCal 3D LUT\"")?;
        writeln!(writer, "LUT_3D_SIZE {}", lut.size)?;
        writeln!(writer)?;

        for rgb in &lut.data {
            writeln!(writer, "{:.6} {:.6} {:.6}", rgb.r, rgb.g, rgb.b)?;
        }

        Ok(())
    }

    /// Export to Autodesk `.3dl` format (10-bit integer values).
    pub fn export_3dl<W: Write>(lut: &Lut3D, writer: &mut W) -> std::io::Result<()> {
        writeln!(writer, "# ArtifexProCal 3D LUT")?;
        writeln!(writer, "# {}", lut.size)?;
        writeln!(writer)?;
        writeln!(writer, "3DMESH")?;
        writeln!(writer, "Mesh {}", lut.size)?;
        writeln!(writer)?;

        for rgb in &lut.data {
            let r = (rgb.r.clamp(0.0, 1.0) * 1023.0).round() as u16;
            let g = (rgb.g.clamp(0.0, 1.0) * 1023.0).round() as u16;
            let b = (rgb.b.clamp(0.0, 1.0) * 1023.0).round() as u16;
            writeln!(writer, "{} {} {}", r, g, b)?;
        }

        writeln!(writer)?;
        writeln!(writer, "# END")?;
        Ok(())
    }
}
