use std::process::{Command, Stdio};
use std::io::Write;
use color_science::types::XYZ;

#[derive(Debug, thiserror::Error)]
pub enum ArgyllError {
    #[error("ArgyllCMS not found: {0}")]
    NotFound(String),
    #[error("Device not found or unavailable")]
    DeviceUnavailable,
    #[error("Measurement failed: {0}")]
    MeasurementFailed(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Timeout waiting for instrument")]
    Timeout,
    #[error("Calibration required: {0}")]
    CalibrationRequired(String),
    #[error("IO error: {0}")]
    Io(String),
}

/// Port number for ArgyllCMS spotread:
/// 1 = i1 Display Pro / ColorMunki Display
/// 2 = i1 Pro 2
pub struct ArgyllPort(pub u8);

impl ArgyllPort {
    pub fn i1_display_pro() -> Self { Self(1) }
    pub fn i1_pro_2() -> Self { Self(2) }
}

/// ArgyllCMS subprocess adapter for macOS.
/// Spawns `spotread` in a PTY (via `expect` script) to drive interactive
/// measurement and parse XYZ results.
pub struct ArgyllMeter {
    port: ArgyllPort,
    model: String,
    connected: bool,
}

impl ArgyllMeter {
    pub fn new(port: ArgyllPort, model: &str) -> Self {
        Self {
            port,
            model: model.to_string(),
            connected: false,
        }
    }

    pub fn connect(&mut self) -> Result<(), ArgyllError> {
        // Verify spotread is available
        match Command::new("spotread").arg("--help").output() {
            Ok(_) => {}
            Err(_) => {
                // Try common install paths
                for path in [
                    "/usr/local/bin/spotread",
                    "/opt/homebrew/bin/spotread",
                    "/usr/bin/spotread",
                ] {
                    if std::path::Path::new(path).exists() {
                        // Found it at this path; we'll use full path in actual calls
                        break;
                    }
                }
            }
        }
        self.connected = true;
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
    }

    pub fn read_xyz(&mut self, _integration_time_ms: u32) -> Result<XYZ, ArgyllError> {
        if !self.connected {
            return Err(ArgyllError::DeviceUnavailable);
        }

        let spotread_path = Self::find_spotread()?;
        let result = self.run_spotread_single(&spotread_path, "-e")?;
        Self::parse_xyz(&result)
    }

    pub fn read_spectrum(&mut self) -> Result<[f64; 36], ArgyllError> {
        if !self.connected {
            return Err(ArgyllError::DeviceUnavailable);
        }

        let spotread_path = Self::find_spotread()?;
        let result = self.run_spotread_single(&spotread_path, "-e -s")?;
        Self::parse_spectrum(&result)
    }

    pub fn initialize(&mut self) -> Result<(), ArgyllError> {
        if !self.connected {
            return Err(ArgyllError::DeviceUnavailable);
        }
        // For i1 Pro 2, initialization is handled by spotread automatically
        // when it prompts for the white reference. No explicit init needed.
        Ok(())
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    fn find_spotread() -> Result<String, ArgyllError> {
        for path in [
            "/usr/local/bin/spotread",
            "/opt/homebrew/bin/spotread",
            "/usr/bin/spotread",
        ] {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }
        // Try PATH
        if Command::new("spotread").arg("--help").output().is_ok() {
            return Ok("spotread".to_string());
        }
        Err(ArgyllError::NotFound(
            "spotread not found. Install ArgyllCMS: brew install argyll-cms".to_string(),
        ))
    }

    /// Spawn spotread via an expect script in a PTY, take one reading, quit.
    fn run_spotread_single(&self,
        spotread: &str,
        extra_flags: &str,
    ) -> Result<String, ArgyllError> {
        // Build expect script inline
        let expect_script = format!(
            r#"
set timeout 20
spawn {spotread} -c {port} {flags}
expect {{
    "Init instrument success" {{ }}
    "Failed to initialise" {{ exit 1 }}
    timeout {{ exit 2 }}
}}
expect {{
    "take a reading" {{ }}
    "needs a calibration" {{ exit 3 }}
    timeout {{ exit 2 }}
}}
sleep 0.2
send "a\r"
expect {{
    -re "Result is XYZ:\\s+X = ([0-9.]+)\\s+Y = ([0-9.]+)\\s+Z = ([0-9.]+)" {{
        puts "READING_START"
        puts "X=$expect_out(1,string) Y=$expect_out(2,string) Z=$expect_out(3,string)"
        puts "READING_END"
    }}
    "Spot read failed" {{
        puts "READING_FAILED"
    }}
    timeout {{ exit 2 }}
}}
send "q\r"
expect eof
"#,
            spotread = spotread,
            port = self.port.0,
            flags = extra_flags,
        );

        let mut child = Command::new("expect")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ArgyllError::Io(format!("Failed to spawn expect: {}", e)))?;

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin
                .write_all(expect_script.as_bytes())
                .map_err(|e| ArgyllError::Io(e.to_string()))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| ArgyllError::Io(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if stdout.contains("READING_FAILED") {
            if stderr.contains("sensor being in the wrong position")
                || stdout.contains("sensor being in the wrong position")
            {
                return Err(ArgyllError::MeasurementFailed(
                    "Ambient filter cap is on — remove it for emissive measurement".to_string(),
                ));
            }
            return Err(ArgyllError::MeasurementFailed(
                "Spot read failed".to_string(),
            ));
        }

        if output.status.code() == Some(3)
            || stderr.contains("needs a calibration")
            || stdout.contains("needs a calibration")
        {
            return Err(ArgyllError::CalibrationRequired(
                "Place instrument on white reference and run initialize()".to_string(),
            ));
        }

        if output.status.code() == Some(2)
            || stdout.contains("timeout")
            || stderr.contains("timeout")
        {
            return Err(ArgyllError::Timeout);
        }

        if output.status.code() == Some(1)
            || stderr.contains("Failed to initialise")
        {
            return Err(ArgyllError::DeviceUnavailable);
        }

        Ok(stdout.to_string())
    }

    fn parse_xyz(output: &str) -> Result<XYZ, ArgyllError> {
        for line in output.lines() {
            if line.starts_with("X=") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let x = parts[0]
                        .trim_start_matches("X=")
                        .parse::<f64>()
                        .map_err(|e| ArgyllError::ParseError(format!("X parse: {}", e)))?;
                    let y = parts[1]
                        .trim_start_matches("Y=")
                        .parse::<f64>()
                        .map_err(|e| ArgyllError::ParseError(format!("Y parse: {}", e)))?;
                    let z = parts[2]
                        .trim_start_matches("Z=")
                        .parse::<f64>()
                        .map_err(|e| ArgyllError::ParseError(format!("Z parse: {}", e)))?;
                    return Ok(XYZ { x, y, z });
                }
            }
        }
        Err(ArgyllError::ParseError(
            "Could not find XYZ in spotread output".to_string(),
        ))
    }

    fn parse_spectrum(_output: &str) -> Result<[f64; 36], ArgyllError> {
        // TODO: implement spectrum parsing from spotread -s output
        // For now return zeros as stub
        Ok([0.0f64; 36])
    }
}
