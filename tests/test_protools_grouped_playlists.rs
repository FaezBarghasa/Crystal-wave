use Crystal_wave::pro_tools::MultiTrackPlaylistGroup;

#[test]
fn test_multi_track_drum_group_playlist_comping() {
    // 10-piece drum kit:
    // 0: Kick In, 1: Kick Out, 2: Snare Top, 3: Snare Bottom, 4: Hi-Hat,
    // 5: Tom 1, 6: Tom 2, 7: Tom 3, 8: Overheads L, 9: Overheads R
    let drum_track_ids: Vec<usize> = (0..10).collect();
    let mut drum_group = MultiTrackPlaylistGroup::new(1, "Drum Tracking Group", drum_track_ids);

    assert_eq!(drum_group.num_tracks(), 10);

    // Record Take 1 (1000 samples per channel, populated with channel-specific test values)
    let take1_buffers: Vec<Vec<f32>> = (0..10).map(|ch| vec![(ch as f32) + 0.1; 1000]).collect();
    let t1_id = drum_group
        .add_take("Take 01 (Groovy)", take1_buffers)
        .expect("Take 1 added");
    assert_eq!(t1_id, 0);

    // Record Take 2 (1000 samples per channel)
    let take2_buffers: Vec<Vec<f32>> = (0..10).map(|ch| vec![(ch as f32) + 0.2; 1000]).collect();
    let t2_id = drum_group
        .add_take("Take 02 (High Energy)", take2_buffers)
        .expect("Take 2 added");
    assert_eq!(t2_id, 1);

    // Record Take 3 (1000 samples per channel)
    let take3_buffers: Vec<Vec<f32>> = (0..10).map(|ch| vec![(ch as f32) + 0.3; 1000]).collect();
    let t3_id = drum_group
        .add_take("Take 03 (Tight Fill)", take3_buffers)
        .expect("Take 3 added");
    assert_eq!(t3_id, 2);

    // 1. Synchronously gang rate takes
    drum_group.gang_rate_take(0, 3).expect("Rate take 1");
    drum_group.gang_rate_take(1, 5).expect("Rate take 2");
    drum_group.gang_rate_take(2, 4).expect("Rate take 3");

    assert_eq!(drum_group.takes[0].rating, 3);
    assert_eq!(drum_group.takes[1].rating, 5);
    assert_eq!(drum_group.takes[2].rating, 4);

    // 2. Gang switch to Take 2 (Best verse take)
    drum_group.gang_switch_take(1).expect("Switch to take 2");
    assert_eq!(drum_group.active_take_idx, 1);

    // Promote intro verse (samples 0..500) from Take 2 into main comp
    drum_group
        .promote_selection_to_main(0, 500)
        .expect("Promote Take 2 verse to main");

    // Gang switch to Take 3 (Best drum fill take)
    drum_group.gang_switch_take(2).expect("Switch to take 3");
    assert_eq!(drum_group.active_take_idx, 2);

    // Promote fill transition (samples 500..1000) from Take 3 into main comp
    drum_group
        .promote_selection_to_main(500, 500)
        .expect("Promote Take 3 fill to main");

    // 3. Verify that ALL 10 channels received the exact synchronous composite
    assert_eq!(drum_group.main_comp_buffers.len(), 10);
    for ch in 0..10 {
        let comp = &drum_group.main_comp_buffers[ch];
        assert_eq!(comp.len(), 1000);

        // First half must match Take 2
        for s in &comp[0..500] {
            assert!((s - ((ch as f32) + 0.2)).abs() < 1e-6);
        }

        // Second half must match Take 3
        for s in &comp[500..1000] {
            assert!((s - ((ch as f32) + 0.3)).abs() < 1e-6);
        }
    }
}
