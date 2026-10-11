#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartToolAction {
    Selector,
    Grabber,
    TrimIn,
    TrimOut,
    FadeIn,
    FadeOut,
    Crossfade,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ClipGeometry {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

#[derive(Debug, Clone)]
pub struct SmartToolContext {
    pub edge_threshold: f32,
    pub corner_threshold: f32,
}

impl Default for SmartToolContext {
    fn default() -> Self {
        Self {
            edge_threshold: 8.0,
            corner_threshold: 14.0,
        }
    }
}

impl SmartToolContext {
    pub fn new(edge_threshold: f32, corner_threshold: f32) -> Self {
        Self {
            edge_threshold,
            corner_threshold,
        }
    }

    /// Resolves the Pro Tools Smart Tool action based on cursor coordinates relative to a clip.
    ///
    /// - **Top corners**: Fade In (left) / Fade Out (right).
    /// - **Left / Right edges**: Trim In (left) / Trim Out (right).
    /// - **Bottom edge between contiguous clips**: Crossfade.
    /// - **Top 50%**: Selector (I-Beam).
    /// - **Bottom 50%**: Grabber (Hand).
    pub fn resolve_action(
        &self,
        clip: &ClipGeometry,
        mouse_x: f32,
        mouse_y: f32,
        has_adjacent_next: bool,
    ) -> SmartToolAction {
        let rel_x = mouse_x - clip.x;
        let rel_y = mouse_y - clip.y;

        // 1. Top corners: Fade triggers
        if rel_y <= self.corner_threshold {
            if rel_x <= self.corner_threshold {
                return SmartToolAction::FadeIn;
            }
            if rel_x >= clip.width - self.corner_threshold {
                return SmartToolAction::FadeOut;
            }
        }

        // 2. Left and Right edges: Trim triggers
        if rel_x <= self.edge_threshold {
            return SmartToolAction::TrimIn;
        }
        if rel_x >= clip.width - self.edge_threshold {
            if has_adjacent_next && rel_y >= clip.height * 0.5 {
                return SmartToolAction::Crossfade;
            }
            return SmartToolAction::TrimOut;
        }

        // 3. Top vs Bottom half: Selector vs Grabber
        if rel_y < clip.height * 0.5 {
            SmartToolAction::Selector
        } else {
            SmartToolAction::Grabber
        }
    }
}

/// Single-keystroke edit commands in Pro Tools Keyboard Focus mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardFocusCommand {
    /// 'B': Split clip at cursor
    SplitAtCursor,
    /// 'A': Trim start of clip to cursor
    TrimStartToCursor,
    /// 'S': Trim end of clip to cursor
    TrimEndToCursor,
    /// 'D': Fade in from start to cursor
    FadeInToCursor,
    /// 'G': Fade out from cursor to end
    FadeOutFromCursor,
    /// 'F': Create crossfade over selection
    CrossfadeSelection,
}

pub fn handle_keyboard_focus_key(key: char) -> Option<KeyboardFocusCommand> {
    match key {
        'b' | 'B' => Some(KeyboardFocusCommand::SplitAtCursor),
        'a' | 'A' => Some(KeyboardFocusCommand::TrimStartToCursor),
        's' | 'S' => Some(KeyboardFocusCommand::TrimEndToCursor),
        'd' | 'D' => Some(KeyboardFocusCommand::FadeInToCursor),
        'g' | 'G' => Some(KeyboardFocusCommand::FadeOutFromCursor),
        'f' | 'F' => Some(KeyboardFocusCommand::CrossfadeSelection),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProToolsClip {
    pub id: usize,
    pub name: String,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub fade_in_ticks: u64,
    pub fade_out_ticks: u64,
}

impl ProToolsClip {
    pub fn new(id: usize, name: &str, start_tick: u64, length_ticks: u64) -> Self {
        Self {
            id,
            name: name.to_string(),
            start_tick,
            length_ticks,
            fade_in_ticks: 0,
            fade_out_ticks: 0,
        }
    }

    pub fn end_tick(&self) -> u64 {
        self.start_tick + self.length_ticks
    }

    /// Executes single-keystroke editing command at `cursor_tick`.
    /// Returns an optional second clip if a split occurred.
    pub fn execute_edit_command(
        &mut self,
        cursor_tick: u64,
        cmd: KeyboardFocusCommand,
        next_id: usize,
    ) -> Option<ProToolsClip> {
        if cursor_tick <= self.start_tick || cursor_tick >= self.end_tick() {
            return None;
        }

        match cmd {
            KeyboardFocusCommand::SplitAtCursor => {
                let first_len = cursor_tick - self.start_tick;
                let second_len = self.length_ticks - first_len;
                self.length_ticks = first_len;

                let second_clip = ProToolsClip::new(
                    next_id,
                    &format!("{}_split", self.name),
                    cursor_tick,
                    second_len,
                );
                Some(second_clip)
            }
            KeyboardFocusCommand::TrimStartToCursor => {
                let delta = cursor_tick - self.start_tick;
                self.start_tick = cursor_tick;
                self.length_ticks = self.length_ticks.saturating_sub(delta);
                None
            }
            KeyboardFocusCommand::TrimEndToCursor => {
                self.length_ticks = cursor_tick - self.start_tick;
                None
            }
            KeyboardFocusCommand::FadeInToCursor => {
                self.fade_in_ticks = cursor_tick - self.start_tick;
                None
            }
            KeyboardFocusCommand::FadeOutFromCursor => {
                self.fade_out_ticks = self.end_tick() - cursor_tick;
                None
            }
            KeyboardFocusCommand::CrossfadeSelection => None,
        }
    }
}
