use Crystal_wave::ui::app::CrystalWaveApp;
use Crystal_wave::ui::layout::{ChannelRackFilter, FlActiveView};
use Crystal_wave::ui::message::UiMessage;

#[test]
fn test_sub_window_floating_geometries_and_focus() {
    let mut app = CrystalWaveApp::new(48000.0);

    // Initial state: default view is Playlist, Channel Rack is visible by default
    assert_eq!(app.layout.active_view, FlActiveView::Playlist);
    assert!(app.fl_windows.channel_rack_visible);
    assert!(!app.fl_windows.piano_roll_visible);
    assert!(!app.fl_windows.mixer_visible);

    // Initial channel rack geometry
    let cr_win = app.layout.sub_windows.channel_rack.clone();
    assert_eq!(cr_win.x, 270.0);
    assert_eq!(cr_win.y, 20.0);
    assert_eq!(cr_win.width, 760.0);
    assert_eq!(cr_win.height, 420.0);
    assert!(cr_win.is_floating);
    assert!(!cr_win.is_maximized);

    // Move Channel Rack window
    app.update(UiMessage::MoveSubWindow {
        view: FlActiveView::ChannelRack,
        dx: 50.0,
        dy: 30.0,
    });
    let cr_moved = app
        .layout
        .sub_windows
        .get_window(FlActiveView::ChannelRack)
        .unwrap();
    assert_eq!(cr_moved.x, 320.0);
    assert_eq!(cr_moved.y, 50.0);
    assert_eq!(
        app.layout.sub_windows.focused_window,
        Some(FlActiveView::ChannelRack)
    );

    // Resize Channel Rack window
    app.update(UiMessage::ResizeSubWindow {
        view: FlActiveView::ChannelRack,
        dw: 100.0,
        dh: 60.0,
    });
    let cr_resized = app
        .layout
        .sub_windows
        .get_window(FlActiveView::ChannelRack)
        .unwrap();
    assert_eq!(cr_resized.width, 860.0);
    assert_eq!(cr_resized.height, 480.0);

    // Set Preset Size
    app.update(UiMessage::SetSubWindowPresetSize {
        view: FlActiveView::ChannelRack,
        width: 1120.0,
        height: 580.0,
    });
    let cr_preset = app
        .layout
        .sub_windows
        .get_window(FlActiveView::ChannelRack)
        .unwrap();
    assert_eq!(cr_preset.width, 1120.0);
    assert_eq!(cr_preset.height, 580.0);

    // Toggle Maximize
    app.update(UiMessage::ToggleSubWindowMaximize(
        FlActiveView::ChannelRack,
    ));
    assert!(app.layout.sub_windows.channel_rack.is_maximized);

    // Restore Maximize
    app.update(UiMessage::ToggleSubWindowMaximize(
        FlActiveView::ChannelRack,
    ));
    assert!(!app.layout.sub_windows.channel_rack.is_maximized);
}

#[test]
fn test_multiple_simultaneous_floating_windows_and_z_ordering() {
    let mut app = CrystalWaveApp::new(48000.0);

    // Open Piano Roll and Mixer so Channel Rack, Piano Roll, and Mixer are all open simultaneously
    app.update(UiMessage::ToggleFlPianoRoll);
    app.update(UiMessage::ToggleFlMixer);

    assert!(app.fl_windows.channel_rack_visible);
    assert!(app.fl_windows.piano_roll_visible);
    assert!(app.fl_windows.mixer_visible);

    // Focus Channel Rack
    app.update(UiMessage::FocusSubWindow(FlActiveView::ChannelRack));
    assert_eq!(
        app.layout.sub_windows.focused_window,
        Some(FlActiveView::ChannelRack)
    );
    let z_cr = app.layout.sub_windows.channel_rack.z_index;

    // Focus Piano Roll - should have higher z_index than Channel Rack
    app.update(UiMessage::FocusSubWindow(FlActiveView::PianoRoll));
    assert_eq!(
        app.layout.sub_windows.focused_window,
        Some(FlActiveView::PianoRoll)
    );
    let z_pr = app.layout.sub_windows.piano_roll.z_index;
    assert!(z_pr > z_cr);

    // Focus Mixer - should have highest z_index
    app.update(UiMessage::FocusSubWindow(FlActiveView::Mixer));
    assert_eq!(
        app.layout.sub_windows.focused_window,
        Some(FlActiveView::Mixer)
    );
    let z_mx = app.layout.sub_windows.mixer.z_index;
    assert!(z_mx > z_pr);

    // Close Piano Roll
    app.update(UiMessage::CloseSubWindow(FlActiveView::PianoRoll));
    assert!(!app.fl_windows.piano_roll_visible);
    assert!(app.fl_windows.mixer_visible);
}

#[test]
fn test_window_arrangements_cascade_and_tile() {
    let mut app = CrystalWaveApp::new(48000.0);

    // Cascade arrangement
    app.update(UiMessage::CascadeSubWindows);
    assert_eq!(app.layout.sub_windows.channel_rack.x, 220.0);
    assert_eq!(app.layout.sub_windows.channel_rack.y, 30.0);
    assert_eq!(app.layout.sub_windows.piano_roll.x, 260.0);
    assert_eq!(app.layout.sub_windows.piano_roll.y, 60.0);
    assert_eq!(app.layout.sub_windows.mixer.x, 300.0);
    assert_eq!(app.layout.sub_windows.mixer.y, 90.0);

    // Tile Horizontal arrangement
    app.update(UiMessage::TileSubWindowsHorizontal);
    assert_eq!(app.layout.sub_windows.channel_rack.x, 240.0);
    assert_eq!(app.layout.sub_windows.channel_rack.y, 20.0);
    assert_eq!(app.layout.sub_windows.piano_roll.x, 850.0);
    assert_eq!(app.layout.sub_windows.piano_roll.y, 20.0);
    assert_eq!(app.layout.sub_windows.mixer.x, 240.0);
    assert_eq!(app.layout.sub_windows.mixer.y, 420.0);

    // Reset arrangement
    app.update(UiMessage::ResetSubWindows);
    assert_eq!(app.layout.sub_windows.channel_rack.x, 270.0);
    assert_eq!(app.layout.sub_windows.channel_rack.y, 20.0);
}

#[test]
fn test_channel_rack_filter_tabs_and_browser_resizing() {
    let mut app = CrystalWaveApp::new(48000.0);

    // Default filter is All
    assert_eq!(app.channel_rack_filter, ChannelRackFilter::All);

    // Switch to Audio filter
    app.update(UiMessage::SetChannelRackFilter(ChannelRackFilter::Audio));
    assert_eq!(app.channel_rack_filter, ChannelRackFilter::Audio);

    // Switch to Unsorted
    app.update(UiMessage::SetChannelRackFilter(ChannelRackFilter::Unsorted));
    assert_eq!(app.channel_rack_filter, ChannelRackFilter::Unsorted);

    // Switch to Automation
    app.update(UiMessage::SetChannelRackFilter(
        ChannelRackFilter::Automation,
    ));
    assert_eq!(app.channel_rack_filter, ChannelRackFilter::Automation);

    // Browser width resizing
    let initial_width = app.browser_width;
    app.update(UiMessage::ResizeBrowser(50.0));
    assert_eq!(app.browser_width, initial_width + 50.0);

    app.update(UiMessage::ResizeBrowser(-100.0));
    assert_eq!(app.browser_width, initial_width - 50.0);
}

#[test]
fn test_view_rendering_with_floating_subwindows() {
    let mut app = CrystalWaveApp::new(48000.0);

    // Open multiple subwindows: Channel Rack, Piano Roll, Mixer, Edison, FPC
    app.update(UiMessage::ToggleFlPianoRoll);
    app.update(UiMessage::ToggleFlMixer);
    app.update(UiMessage::ToggleFlEdison);
    app.update(UiMessage::ToggleFlFpc);

    // Render main view - verifies full MDI compositor builds successfully without panics
    let _element = app.view();
}
