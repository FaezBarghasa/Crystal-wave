use Crystal_wave::pro_tools::{MemoryLocation, MemoryLocationBank};

#[test]
fn test_protools_memory_location_storage_and_recall() {
    let mut bank = MemoryLocationBank::new();

    // 1. Create Location 1: "Verse 1 Vocal Tracking"
    let loc1 = MemoryLocation::new(1, "Verse 1 Vocal", 960 * 8)
        .with_zoom(2.5)
        .with_track_visibility(vec![true, true, false, false, true])
        .with_pre_post_roll(960 * 2, 960 * 2);

    // 2. Create Location 2: "Bridge Drum Edit"
    let loc2 = MemoryLocation::new(2, "Bridge Drum Edit", 960 * 32)
        .with_zoom(8.0)
        .with_track_visibility(vec![false, false, true, true, false])
        .with_pre_post_roll(0, 0);

    // 3. Create Location 5: "Mastering Limiter Check"
    let loc5 = MemoryLocation::new(5, "Chorus 2 Climax", 960 * 48)
        .with_zoom(1.0);

    bank.store(loc1).expect("Store loc 1");
    bank.store(loc2).expect("Store loc 2");
    bank.store(loc5).expect("Store loc 5");

    // Boundary checks: 0 or >999 rejected
    let loc_zero = MemoryLocation::new(0, "Invalid Zero", 0);
    assert!(bank.store(loc_zero).is_err());
    let loc_over = MemoryLocation::new(1000, "Invalid 1000", 0);
    assert!(bank.store(loc_over).is_err());

    // 4. Recall Location 1
    let recalled_1 = bank.recall(1).expect("Recall 1");
    assert_eq!(recalled_1.name, "Verse 1 Vocal");
    assert_eq!(recalled_1.tick, 960 * 8);
    assert_eq!(recalled_1.zoom_factor, 2.5);
    assert_eq!(recalled_1.track_visibility_mask, vec![true, true, false, false, true]);
    assert_eq!(recalled_1.pre_roll_ticks, 960 * 2);

    // 5. Recall Location 2
    let recalled_2 = bank.recall(2).expect("Recall 2");
    assert_eq!(recalled_2.name, "Bridge Drum Edit");
    assert_eq!(recalled_2.zoom_factor, 8.0);
    assert_eq!(recalled_2.track_visibility_mask, vec![false, false, true, true, false]);

    // 6. Non-existent recall
    assert!(bank.recall(3).is_none());

    // 7. Ordered list
    let all = bank.list_all();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].number, 1);
    assert_eq!(all[1].number, 2);
    assert_eq!(all[2].number, 5);

    // 8. Delete
    let deleted = bank.delete(2);
    assert!(deleted.is_some());
    assert_eq!(bank.recall(2), None);
    assert_eq!(bank.list_all().len(), 2);
}
