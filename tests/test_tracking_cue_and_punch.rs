use Crystal_wave::audio::punch_engine::{OnTheFlyPunchEngine, PunchState};
use Crystal_wave::mixing::cue_matrix::CueMatrix;
use Crystal_wave::pro_tools::vca_group::{TrackSpillState, VcaManager};

#[test]
fn test_cue_matrix_routing_and_rendering() {
    let mut matrix = CueMatrix::new(4);
    assert_eq!(matrix.cue_mixes.len(), 2);
    assert!(matrix.cue_mixes[0].click_enabled);
    assert!(!matrix.cue_mixes[1].click_enabled);

    // Set send level for Channel 0 to Cue 0
    let res = matrix.set_send_level(0, 0, 0.0); // 0 dB = unity gain
    assert!(res.is_ok());

    let buf_left = vec![0.5f32; 128];
    let buf_right = vec![0.5f32; 128];
    let channels = [(&buf_left[..], &buf_right[..])];
    let click = vec![1.0f32; 128];

    let mut out_l = vec![0.0f32; 128];
    let mut out_r = vec![0.0f32; 128];

    // Render Cue 0 (with click)
    matrix.render_cue_mix(0, &channels, &click, &mut out_l, &mut out_r);
    // Click should be injected into out_l and out_r
    assert!(out_l[0] > 0.0);
    assert!(out_r[0] > 0.0);

    // Render Cue 1 (without click)
    let mut out_l1 = vec![0.0f32; 128];
    let mut out_r1 = vec![0.0f32; 128];
    matrix.render_cue_mix(1, &channels, &click, &mut out_l1, &mut out_r1);
    // Should have audio from channels, but without click gain addition
    assert!(out_l1[0] < out_l[0]);
}

#[test]
fn test_on_the_fly_punch_engine_crossfade() {
    let mut engine = OnTheFlyPunchEngine::new(32);
    assert_eq!(engine.state, PunchState::PlaybackOnly);

    engine.trigger_punch_in(50);
    assert_eq!(engine.state, PunchState::RecordingActive);
    assert_eq!(engine.last_punch_in_sample, Some(50));

    let original = vec![1.0f32; 100];
    let new_take = vec![0.0f32; 100];
    let mut output = vec![0.0f32; 100];

    engine.blend_punch_boundary(&original, &new_take, 50, true, &mut output);

    // Before punch boundary, should be original
    assert_eq!(output[20], 1.0);
    // Mid crossfade, should be blend
    assert!(output[60] > 0.0 && output[60] < 1.0);
    // After crossfade, should be new take (0.0)
    assert_eq!(output[90], 0.0);

    // Punch out
    engine.trigger_punch_out(80);
    assert_eq!(engine.state, PunchState::PlaybackOnly);
    assert_eq!(engine.last_punch_out_sample, Some(80));
}

#[test]
fn test_vca_spill_isolation() {
    let mut vca_mgr = VcaManager::new();
    let drums_vca = vca_mgr.create_vca("Drums VCA");

    // Assign tracks 1 and 2 to Drums VCA
    assert!(vca_mgr.assign_track_to_vca(drums_vca, 1).is_ok());
    assert!(vca_mgr.assign_track_to_vca(drums_vca, 2).is_ok());

    // Before spill is engaged, all tracks are in normal state
    assert_eq!(vca_mgr.track_spill_state(1), TrackSpillState::Normal);
    assert_eq!(vca_mgr.track_spill_state(3), TrackSpillState::Normal);

    // Engage spill on Drums VCA
    vca_mgr.toggle_spill(drums_vca);
    assert_eq!(vca_mgr.active_spill_vca, Some(drums_vca));

    // Assigned tracks are isolated
    assert_eq!(vca_mgr.track_spill_state(1), TrackSpillState::SpillIsolated);
    assert_eq!(vca_mgr.track_spill_state(2), TrackSpillState::SpillIsolated);
    // Non-assigned tracks are dimmed
    assert_eq!(vca_mgr.track_spill_state(3), TrackSpillState::SpillDimmed);

    // Toggle spill off
    vca_mgr.toggle_spill(drums_vca);
    assert_eq!(vca_mgr.active_spill_vca, None);
    assert_eq!(vca_mgr.track_spill_state(1), TrackSpillState::Normal);
    assert_eq!(vca_mgr.track_spill_state(3), TrackSpillState::Normal);
}
