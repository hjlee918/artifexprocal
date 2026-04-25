use hal::traits::{PatternGenerator, PatternGeneratorExt};
use hal::error::PatternGenError;
use color_science::types::RGB;

struct MockPg;

impl PatternGenerator for MockPg {
    fn connect(&mut self) -> Result<(), PatternGenError> { Ok(()) }
    fn disconnect(&mut self) {}
    fn display_patch(&mut self, _color: &RGB) -> Result<(), PatternGenError> { Ok(()) }
}

impl PatternGeneratorExt for MockPg {
    fn display_pattern(&mut self, _name: &str) -> Result<(), PatternGenError> { Ok(()) }
    fn list_patterns(&self) -> Vec<String> { vec!["test".to_string()] }
}

#[test]
fn test_pattern_generator_ext_exists() {
    let mut pg = MockPg;
    assert!(pg.display_pattern("21-Point Grayscale").is_ok());
    assert_eq!(pg.list_patterns().len(), 1);
}
