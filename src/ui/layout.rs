//! Docking, Panel Layout and Multi-Pane Window Management
//!
//! Clean-room implementation of FL Studio's modular multi-window layout system:
//! Top toolbar, detachable panels (Channel Rack, Piano Roll, Playlist, Mixer, Browser,
//! Edison, FPC, Slicex, Patcher) with instant F-key focus switching.

use serde::{Deserialize, Serialize};

/// Active primary view in the main workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FlActiveView {
    #[default]
    Playlist, // F5
    ChannelRack, // F6
    PianoRoll,   // F7
    Browser,     // Alt+F8
    Mixer,       // F9
    Edison,      // Audio Editor
    Fpc,         // 16-pad drum machine
    Slicex,      // Beat slicer
    Patcher,     // Modular node routing
}

impl FlActiveView {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Playlist => "Playlist [F5]",
            Self::ChannelRack => "Channel Rack [F6]",
            Self::PianoRoll => "Piano Roll [F7]",
            Self::Browser => "Browser [Alt+F8]",
            Self::Mixer => "Mixer [F9]",
            Self::Edison => "Edison Editor",
            Self::Fpc => "FPC Drums",
            Self::Slicex => "Slicex Slicer",
            Self::Patcher => "Modular Patcher",
        }
    }
}

/// Identifiers for major DAW functional panels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    Browser,
    Inspector,
    Arrangement,
    EditorDrawer,
    Mixer,
    ChannelRack,
    Patcher,
    PluginRack,
    Edison,
    Fpc,
    Slicex,
}

/// Docking anchor for panels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DockPosition {
    #[default]
    Center,
    Left,
    Right,
    Bottom,
    Float,
}

/// Bottom editor drawer sub-views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DrawerTab {
    #[default]
    PianoRoll,
    StepSequencer,
    Automation,
    Mixer,
}

impl DrawerTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PianoRoll => "Piano Roll [F7]",
            Self::StepSequencer => "Step Sequencer [F6]",
            Self::Automation => "Automation",
            Self::Mixer => "Mixer [F9]",
        }
    }
}

/// Individual panel configuration and state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelState {
    pub id: PanelId,
    pub visible: bool,
    pub dock: DockPosition,
    pub split_ratio: f32,
    pub is_collapsed: bool,
    pub float_pos: (f32, f32),
    pub float_size: (f32, f32),
}

impl PanelState {
    pub fn new(id: PanelId, dock: DockPosition, split_ratio: f32, visible: bool) -> Self {
        Self {
            id,
            visible,
            dock,
            split_ratio,
            is_collapsed: false,
            float_pos: (100.0, 100.0),
            float_size: (600.0, 400.0),
        }
    }
}

/// Active menu dropdown in top menu bar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveMenuDropdown {
    File,
    Edit,
    Add,
    Patterns,
    View,
    Options,
    Tools,
    Help,
}

/// Individual sub-window floating and geometry state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubWindowState {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_floating: bool,
    pub is_maximized: bool,
    pub z_index: u32,
}

impl SubWindowState {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            is_floating: true,
            is_maximized: false,
            z_index: 0,
        }
    }
}

/// Floating Sub-Windows Manager for FL Studio MDI Workspace
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubWindowManager {
    pub channel_rack: SubWindowState,
    pub piano_roll: SubWindowState,
    pub mixer: SubWindowState,
    pub edison: SubWindowState,
    pub fpc: SubWindowState,
    pub slicex: SubWindowState,
    pub patcher: SubWindowState,
    pub focused_window: Option<FlActiveView>,
    pub next_z: u32,
}

impl Default for SubWindowManager {
    fn default() -> Self {
        Self {
            channel_rack: SubWindowState::new(270.0, 20.0, 760.0, 420.0),
            piano_roll: SubWindowState::new(300.0, 50.0, 900.0, 500.0),
            mixer: SubWindowState::new(260.0, 70.0, 960.0, 500.0),
            edison: SubWindowState::new(280.0, 40.0, 840.0, 480.0),
            fpc: SubWindowState::new(280.0, 30.0, 800.0, 480.0),
            slicex: SubWindowState::new(280.0, 30.0, 820.0, 480.0),
            patcher: SubWindowState::new(270.0, 40.0, 860.0, 500.0),
            focused_window: Some(FlActiveView::ChannelRack),
            next_z: 1,
        }
    }
}

impl SubWindowManager {
    pub fn get_window_mut(&mut self, view: FlActiveView) -> Option<&mut SubWindowState> {
        match view {
            FlActiveView::ChannelRack => Some(&mut self.channel_rack),
            FlActiveView::PianoRoll => Some(&mut self.piano_roll),
            FlActiveView::Mixer => Some(&mut self.mixer),
            FlActiveView::Edison => Some(&mut self.edison),
            FlActiveView::Fpc => Some(&mut self.fpc),
            FlActiveView::Slicex => Some(&mut self.slicex),
            FlActiveView::Patcher => Some(&mut self.patcher),
            _ => None,
        }
    }

    pub fn get_window(&self, view: FlActiveView) -> Option<&SubWindowState> {
        match view {
            FlActiveView::ChannelRack => Some(&self.channel_rack),
            FlActiveView::PianoRoll => Some(&self.piano_roll),
            FlActiveView::Mixer => Some(&self.mixer),
            FlActiveView::Edison => Some(&self.edison),
            FlActiveView::Fpc => Some(&self.fpc),
            FlActiveView::Slicex => Some(&self.slicex),
            FlActiveView::Patcher => Some(&self.patcher),
            _ => None,
        }
    }

    pub fn focus(&mut self, view: FlActiveView) {
        self.focused_window = Some(view);
        self.next_z += 1;
        let z = self.next_z;
        if let Some(w) = self.get_window_mut(view) {
            w.z_index = z;
        }
    }

    pub fn move_window(&mut self, view: FlActiveView, dx: f32, dy: f32) {
        if let Some(w) = self.get_window_mut(view) {
            w.x = (w.x + dx).max(0.0);
            w.y = (w.y + dy).max(0.0);
        }
        self.focus(view);
    }

    pub fn resize_window(&mut self, view: FlActiveView, dw: f32, dh: f32) {
        if let Some(w) = self.get_window_mut(view) {
            w.width = (w.width + dw).clamp(380.0, 1920.0);
            w.height = (w.height + dh).clamp(240.0, 1080.0);
        }
        self.focus(view);
    }

    pub fn toggle_maximize(&mut self, view: FlActiveView) {
        if let Some(w) = self.get_window_mut(view) {
            w.is_maximized = !w.is_maximized;
        }
        self.focus(view);
    }

    pub fn toggle_floating(&mut self, view: FlActiveView) {
        if let Some(w) = self.get_window_mut(view) {
            w.is_floating = !w.is_floating;
        }
        self.focus(view);
    }

    pub fn cascade(&mut self) {
        self.channel_rack = SubWindowState::new(220.0, 30.0, 760.0, 420.0);
        self.piano_roll = SubWindowState::new(260.0, 60.0, 880.0, 480.0);
        self.mixer = SubWindowState::new(300.0, 90.0, 920.0, 500.0);
        self.edison = SubWindowState::new(340.0, 120.0, 840.0, 460.0);
        self.fpc = SubWindowState::new(320.0, 100.0, 800.0, 460.0);
        self.slicex = SubWindowState::new(300.0, 80.0, 820.0, 460.0);
        self.patcher = SubWindowState::new(280.0, 70.0, 860.0, 480.0);
    }

    pub fn tile_horizontal(&mut self) {
        self.channel_rack = SubWindowState::new(240.0, 20.0, 600.0, 380.0);
        self.piano_roll = SubWindowState::new(850.0, 20.0, 750.0, 380.0);
        self.mixer = SubWindowState::new(240.0, 420.0, 1360.0, 380.0);
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Filter group tabs for FL Studio Channel Rack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChannelRackFilter {
    #[default]
    All,
    Audio,
    Unsorted,
    Automation,
}

impl ChannelRackFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::Audio => "AUDIO",
            Self::Unsorted => "UNSORTED",
            Self::Automation => "AUTOMATION",
        }
    }
}

/// Complete Workspace Layout State
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutState {
    pub active_view: FlActiveView,
    pub secondary_view: Option<FlActiveView>,
    pub active_dropdown: Option<ActiveMenuDropdown>,
    pub sub_windows: SubWindowManager,
    pub browser: PanelState,
    pub inspector: PanelState,
    pub bottom_drawer: PanelState,
    pub mixer_panel: PanelState,
    pub channel_rack_panel: PanelState,
    pub patcher_panel: PanelState,
    pub active_drawer_tab: DrawerTab,
    pub detached_to_os_window: Vec<PanelId>,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            active_view: FlActiveView::Playlist,
            secondary_view: None,
            active_dropdown: None,
            sub_windows: SubWindowManager::default(),
            browser: PanelState::new(PanelId::Browser, DockPosition::Left, 0.20, true),
            inspector: PanelState::new(PanelId::Inspector, DockPosition::Left, 0.18, false),
            bottom_drawer: PanelState::new(PanelId::EditorDrawer, DockPosition::Bottom, 0.35, true),
            mixer_panel: PanelState::new(PanelId::Mixer, DockPosition::Bottom, 0.40, false),
            channel_rack_panel: PanelState::new(
                PanelId::ChannelRack,
                DockPosition::Float,
                0.30,
                true,
            ),
            patcher_panel: PanelState::new(PanelId::Patcher, DockPosition::Float, 0.50, false),
            active_drawer_tab: DrawerTab::PianoRoll,
            detached_to_os_window: Vec::new(),
        }
    }
}

impl LayoutState {
    /// Toggle visibility of a panel
    pub fn toggle_panel(&mut self, panel: PanelId) {
        match panel {
            PanelId::Browser => self.browser.visible = !self.browser.visible,
            PanelId::Inspector => self.inspector.visible = !self.inspector.visible,
            PanelId::EditorDrawer => self.bottom_drawer.visible = !self.bottom_drawer.visible,
            PanelId::Mixer => self.mixer_panel.visible = !self.mixer_panel.visible,
            PanelId::ChannelRack => {
                self.channel_rack_panel.visible = !self.channel_rack_panel.visible
            }
            PanelId::Patcher => self.patcher_panel.visible = !self.patcher_panel.visible,
            _ => {}
        }
    }

    /// Set dock position for a panel
    pub fn set_dock(&mut self, panel: PanelId, dock: DockPosition) {
        let p = match panel {
            PanelId::Browser => &mut self.browser,
            PanelId::Inspector => &mut self.inspector,
            PanelId::EditorDrawer => &mut self.bottom_drawer,
            PanelId::Mixer => &mut self.mixer_panel,
            PanelId::ChannelRack => &mut self.channel_rack_panel,
            PanelId::Patcher => &mut self.patcher_panel,
            _ => return,
        };
        p.dock = dock;
    }

    /// Detach panel into a floating / OS level window
    pub fn detach_to_window(&mut self, panel: PanelId) {
        if !self.detached_to_os_window.contains(&panel) {
            self.detached_to_os_window.push(panel);
            self.set_dock(panel, DockPosition::Float);
        }
    }

    /// Re-dock panel back into main application layout
    pub fn redock_to_main(&mut self, panel: PanelId, dock: DockPosition) {
        self.detached_to_os_window.retain(|&p| p != panel);
        self.set_dock(panel, dock);
    }

    /// Check if panel is detached
    pub fn is_detached(&self, panel: PanelId) -> bool {
        self.detached_to_os_window.contains(&panel)
    }

    /// Switch active drawer tab
    pub fn set_drawer_tab(&mut self, tab: DrawerTab) {
        self.active_drawer_tab = tab;
        self.bottom_drawer.visible = true;
    }
}
