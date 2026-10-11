use Crystal_wave::pro_tools::{
    ClipGeometry, KeyboardFocusCommand, ProToolsClip, SmartToolAction, SmartToolContext,
    handle_keyboard_focus_key,
};

#[test]
fn test_smart_tool_hotspot_resolution() {
    let ctx = SmartToolContext::default();
    let clip = ClipGeometry::new(100.0, 50.0, 200.0, 100.0); // x:100..300, y:50..150

    // 1. Top-left corner -> Fade In
    let action_fade_in = ctx.resolve_action(&clip, 104.0, 54.0, false);
    assert_eq!(action_fade_in, SmartToolAction::FadeIn);

    // 2. Top-right corner -> Fade Out
    let action_fade_out = ctx.resolve_action(&clip, 296.0, 54.0, false);
    assert_eq!(action_fade_out, SmartToolAction::FadeOut);

    // 3. Left edge (middle height) -> Trim In
    let action_trim_in = ctx.resolve_action(&clip, 103.0, 100.0, false);
    assert_eq!(action_trim_in, SmartToolAction::TrimIn);

    // 4. Right edge (top half) -> Trim Out
    let action_trim_out = ctx.resolve_action(&clip, 298.0, 80.0, false);
    assert_eq!(action_trim_out, SmartToolAction::TrimOut);

    // 5. Right edge (bottom half with adjacent clip) -> Crossfade
    let action_crossfade = ctx.resolve_action(&clip, 298.0, 120.0, true);
    assert_eq!(action_crossfade, SmartToolAction::Crossfade);

    // 6. Upper half interior -> Selector (I-Beam)
    let action_selector = ctx.resolve_action(&clip, 200.0, 75.0, false);
    assert_eq!(action_selector, SmartToolAction::Selector);

    // 7. Lower half interior -> Grabber (Hand)
    let action_grabber = ctx.resolve_action(&clip, 200.0, 125.0, false);
    assert_eq!(action_grabber, SmartToolAction::Grabber);
}

#[test]
fn test_keyboard_focus_single_key_shortcuts() {
    assert_eq!(
        handle_keyboard_focus_key('b'),
        Some(KeyboardFocusCommand::SplitAtCursor)
    );
    assert_eq!(
        handle_keyboard_focus_key('B'),
        Some(KeyboardFocusCommand::SplitAtCursor)
    );
    assert_eq!(
        handle_keyboard_focus_key('a'),
        Some(KeyboardFocusCommand::TrimStartToCursor)
    );
    assert_eq!(
        handle_keyboard_focus_key('s'),
        Some(KeyboardFocusCommand::TrimEndToCursor)
    );
    assert_eq!(
        handle_keyboard_focus_key('d'),
        Some(KeyboardFocusCommand::FadeInToCursor)
    );
    assert_eq!(
        handle_keyboard_focus_key('g'),
        Some(KeyboardFocusCommand::FadeOutFromCursor)
    );
    assert_eq!(
        handle_keyboard_focus_key('f'),
        Some(KeyboardFocusCommand::CrossfadeSelection)
    );
    assert_eq!(handle_keyboard_focus_key('z'), None);
}

#[test]
fn test_clip_editing_execution() {
    let mut clip = ProToolsClip::new(1, "Lead Vocal", 1000, 2000); // 1000..3000
    assert_eq!(clip.end_tick(), 3000);

    // 1. Split at cursor tick 1800
    let split_clip = clip
        .execute_edit_command(1800, KeyboardFocusCommand::SplitAtCursor, 2)
        .expect("Split must succeed");

    assert_eq!(clip.start_tick, 1000);
    assert_eq!(clip.length_ticks, 800);
    assert_eq!(clip.end_tick(), 1800);

    assert_eq!(split_clip.id, 2);
    assert_eq!(split_clip.start_tick, 1800);
    assert_eq!(split_clip.length_ticks, 1200);
    assert_eq!(split_clip.end_tick(), 3000);

    // 2. Trim start of first clip to cursor 1200 ('A')
    let res = clip.execute_edit_command(1200, KeyboardFocusCommand::TrimStartToCursor, 3);
    assert!(res.is_none());
    assert_eq!(clip.start_tick, 1200);
    assert_eq!(clip.length_ticks, 600);

    // 3. Trim end of first clip to cursor 1600 ('S')
    let res = clip.execute_edit_command(1600, KeyboardFocusCommand::TrimEndToCursor, 4);
    assert!(res.is_none());
    assert_eq!(clip.length_ticks, 400);
    assert_eq!(clip.end_tick(), 1600);

    // 4. Fade in to cursor 1300 ('D')
    clip.execute_edit_command(1300, KeyboardFocusCommand::FadeInToCursor, 5);
    assert_eq!(clip.fade_in_ticks, 100);

    // 5. Fade out from cursor 1500 ('G')
    clip.execute_edit_command(1500, KeyboardFocusCommand::FadeOutFromCursor, 6);
    assert_eq!(clip.fade_out_ticks, 100);
}
