use Crystal_wave::plugins::omid_adapter::DynamicPluginHost;
use Crystal_wave::plugins::rust_sdk::CrystalPlugin;
use Crystal_wave::plugins::sdk_ffi::CrystalPluginVTable;
use Crystal_wave::plugins::AudioPlugin;
use Crystal_wave::export_crystal_plugin;
use omid::event::{OmidEventType, OmidFlags};
use omid::OmidPacket;

#[derive(Default)]
struct TestOmidPlugin {
    pub gain: f32,
    pub last_received_packet: Option<OmidPacket>,
}

impl CrystalPlugin for TestOmidPlugin {
    fn name(&self) -> &'static str { "Test Omid Synth" }
    fn vendor(&self) -> &'static str { "OmidAudio" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> u32 { 9 } // Synth
    fn unique_id(&self) -> u32 { 7777 }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l *= self.gain;
            *r *= self.gain;
        }
    }

    fn parameter_count(&self) -> usize { 1 }
    fn parameter_name(&self, _index: usize) -> &'static str { "Master Gain" }
    fn get_parameter(&self, _index: usize) -> f32 { self.gain }
    fn set_parameter(&mut self, _index: usize, val: f32) { self.gain = val; }

    fn on_omid_packet(&mut self, packet: OmidPacket) {
        self.last_received_packet = Some(packet);
        if packet.is_raw_data() {
            // 12-bit ADC fader control
            self.gain = packet.payload_as_normalized_f32(12);
        }
    }

    fn poll_omid_response(&mut self) -> Option<OmidPacket> {
        // Echo back with modified event type
        self.last_received_packet.map(|pkt| {
            OmidPacket::new(pkt.object_id, OmidEventType::RelativeChange as u8, 0, 42)
        })
    }
}

export_crystal_plugin!(TestOmidPlugin);

#[test]
fn test_dynamic_plugin_sdk_lifecycle_and_omid_packets() {
    let vtable: CrystalPluginVTable = crystal_plugin_get_vtable();
    let mut host = DynamicPluginHost::new_from_vtable(vtable, 48000.0)
        .expect("Failed to initialize dynamic plugin host");

    assert_eq!(host.descriptor().name, "Crystal Plugin");
    assert_eq!(host.descriptor().unique_id, 8888);

    // 1. Audio Processing Check
    let mut left = vec![1.0; 64];
    let mut right = vec![1.0; 64];

    // Initial gain is 0.0, output should be silent
    host.process(&mut left, &mut right, None);
    assert_eq!(left[0], 0.0);
    assert_eq!(right[0], 0.0);

    // 2. Dispatch 12-bit ADC Omid Packet (value 2048 -> normalized 0.5)
    let flags = OmidFlags::new(false, false, false, 0);
    let packet = OmidPacket::new_adc12(101, OmidEventType::AbsoluteChange, flags, 2048);
    host.dispatch_omid_packet(packet);

    // Reprocess with updated gain from OMID
    let mut left2 = vec![1.0; 64];
    let mut right2 = vec![1.0; 64];
    host.process(&mut left2, &mut right2, None);
    assert!((left2[0] - 0.5).abs() < 1e-3);
    assert!((right2[0] - 0.5).abs() < 1e-3);

    // 3. Poll Omid Response Packet
    let response_packet = host.poll_omid_response();
    assert!(response_packet.is_some());
    let resp = response_packet.unwrap();
    assert_eq!(resp.object_id, 101);
    assert_eq!(resp.payload, 42);
}
