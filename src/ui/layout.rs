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
    Playlist,    // F5
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

/// Complete Workspace Layout State
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutState {
    pub active_view: FlActiveView,
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
            active_drawer_tab: DrawerTab::StepSequencer,
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
