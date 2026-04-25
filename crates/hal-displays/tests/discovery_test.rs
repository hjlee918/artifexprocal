use hal_displays::discovery::*;

#[test]
fn test_ssdp_message_format() {
    let msg = SsdpDiscovery::build_msearch();
    assert!(msg.contains("M-SEARCH"));
    assert!(msg.contains("239.255.255.250"));
    assert!(msg.contains("urn:lge-com:service:webos-second-screen:1"));
}

#[test]
fn test_parse_ssdp_response() {
    let response = "HTTP/1.1 200 OK\r\nLOCATION: http://192.168.1.100:3000\r\n\r\n";
    let ip = SsdpDiscovery::parse_location(response).unwrap();
    assert_eq!(ip, "192.168.1.100");
}

#[test]
fn test_parse_ssdp_response_https() {
    let response = "HTTP/1.1 200 OK\r\nLOCATION: https://192.168.1.101:3001/ws\r\n\r\n";
    let ip = SsdpDiscovery::parse_location(response).unwrap();
    assert_eq!(ip, "192.168.1.101");
}
