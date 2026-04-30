use hal_patterns::patterns_catalog::*;

#[test]
fn test_catalog_contains_grayscale() {
    let catalog = ted_disk_patterns();
    assert!(catalog.contains(&"21-Point Grayscale"));
}

#[test]
fn test_catalog_contains_color_checker() {
    let catalog = ted_disk_patterns();
    assert!(catalog.contains(&"Color Checker Classic (24 Colors)"));
}

#[test]
fn test_catalog_size() {
    assert!(ted_disk_patterns().len() >= 20);
}
