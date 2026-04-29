use std::net::UdpSocket;
use std::time::Duration;

pub struct SsdpDiscovery;

impl SsdpDiscovery {
    pub fn build_msearch() -> String {
        "M-SEARCH * HTTP/1.1\r\n\
         HOST: 239.255.255.250:1900\r\n\
         MAN: \"ssdp:discover\"\r\n\
         ST: urn:lge-com:service:webos-second-screen:1\r\n\
         MX: 2\r\n\r\n"
            .to_string()
    }

    pub fn discover(timeout_ms: u64) -> Result<Vec<String>, String> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| format!("Bind failed: {}", e))?;
        socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))
            .map_err(|e| format!("Timeout setup: {}", e))?;

        let msearch = Self::build_msearch();
        socket.send_to(msearch.as_bytes(), "239.255.255.250:1900")
            .map_err(|e| format!("Send failed: {}", e))?;

        let mut ips = Vec::new();
        let mut buf = [0u8; 1024];
        while let Ok((len, _)) = socket.recv_from(&mut buf) {
            let resp = String::from_utf8_lossy(&buf[..len]);
            if let Some(ip) = Self::parse_location(&resp) {
                if !ips.contains(&ip) {
                    ips.push(ip);
                }
            }
        }
        Ok(ips)
    }

    pub fn parse_location(response: &str) -> Option<String> {
        for line in response.lines() {
            if line.to_uppercase().starts_with("LOCATION:") {
                let rest = line["LOCATION:".len()..].trim();
                let after_scheme = rest.split("://").nth(1)?;
                let host_port = after_scheme.split('/').next()?;
                let ip = host_port.split(':').next()?;
                return Some(ip.to_string());
            }
        }
        None
    }
}
