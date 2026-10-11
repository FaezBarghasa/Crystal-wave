use crate::ui::app::CrystalWaveApp;
use crate::ui::message::UiMessage;

pub struct UiSessionDriver {
    pub app: CrystalWaveApp,
}

impl UiSessionDriver {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            app: CrystalWaveApp::new(sample_rate),
        }
    }

    pub fn dispatch(&mut self, message: UiMessage) -> &mut Self {
        self.app.update(message);
        self
    }

    pub fn dispatch_shortcut(&mut self, key: char) -> &mut Self {
        match key {
            'i' | 'I' => self.dispatch(UiMessage::ToggleInspector),
            'e' | 'E' => self.dispatch(UiMessage::ToggleEditorDrawer),
            'x' | 'X' => self.dispatch(UiMessage::ToggleMixerDrawer),
            'o' | 'O' => self.dispatch(UiMessage::ToggleLoopBrowser),
            'f' | 'F' => self.dispatch(UiMessage::ToggleFileBrowser),
            'y' | 'Y' => self.dispatch(UiMessage::ToggleLibraryBrowser),
            ' ' => {
                if self.app.is_playing {
                    self.dispatch(UiMessage::TransportPause)
                } else {
                    self.dispatch(UiMessage::TransportPlay)
                }
            }
            'r' | 'R' => self.dispatch(UiMessage::TransportRecord),
            '\n' => self.dispatch(UiMessage::TransportStop),
            _ => self,
        }
    }

    pub fn assert_inspector_open(&self, expected: bool) -> &Self {
        assert_eq!(self.app.workspace.inspector_open, expected);
        self
    }

    pub fn assert_editor_open(&self, expected: bool) -> &Self {
        assert_eq!(self.app.workspace.editor_drawer_open, expected);
        self
    }

    pub fn assert_mixer_open(&self, expected: bool) -> &Self {
        assert_eq!(self.app.workspace.mixer_drawer_open, expected);
        self
    }

    pub fn assert_playing(&self, expected: bool) -> &Self {
        assert_eq!(self.app.is_playing, expected);
        self
    }

    pub fn assert_recording(&self, expected: bool) -> &Self {
        assert_eq!(self.app.is_recording, expected);
        self
    }

    pub fn assert_playhead(&self, expected_tick: u64) -> &Self {
        assert_eq!(self.app.playhead_tick, expected_tick);
        self
    }
}
