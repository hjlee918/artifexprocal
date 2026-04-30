use calibration_engine::events::EventChannel;
use calibration_core::state::CalibrationEvent;
use color_science::types::RGB;

#[test]
fn test_event_send_and_receive() {
    let channel = EventChannel::new(16);
    let mut rx = channel.subscribe();

    channel.send(CalibrationEvent::PatchDisplayed {
        patch_index: 0,
        rgb: RGB { r: 1.0, g: 0.0, b: 0.0 },
    });

    let event = rx.try_recv().unwrap();
    assert!(matches!(event, CalibrationEvent::PatchDisplayed { patch_index: 0, .. }));
}
