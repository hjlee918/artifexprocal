use hal_patterns::xml_protocol::*;

#[test]
fn test_xml_patch_serialize() {
    let patch = XmlPatch { r: 128, g: 64, b: 32 };
    let xml = patch.to_xml();
    assert!(xml.contains("<patch>"));
    assert!(xml.contains("<r>128</r>"));
    assert!(xml.contains("<g>64</g>"));
    assert!(xml.contains("<b>32</b>"));
}

#[test]
fn test_xml_pattern_serialize() {
    let pat = XmlPattern {
        name: "21-Point Grayscale".to_string(),
        chapter: 1,
    };
    let xml = pat.to_xml();
    assert!(xml.contains("<name>21-Point Grayscale</name>"));
    assert!(xml.contains("<chapter>1</chapter>"));
}

#[test]
fn test_xml_black_patch() {
    let patch = XmlPatch::black();
    let xml = patch.to_xml();
    assert!(xml.contains("<r>0</r>"));
    assert!(xml.contains("<g>0</g>"));
    assert!(xml.contains("<b>0</b>"));
}
