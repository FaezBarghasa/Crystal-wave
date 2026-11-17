//! Interactive 2D Graphical Piano Roll Canvas Program (FL Studio & Apple Logic Pro Style)
//!
//! Features:
//! - 128-Key vertical piano rake (C-1 to G9) with white/black key geometry and octave markers
//! - 2D Note Matrix with sub-beat (16th), beat, and bar grid lines
//! - Real-time Note Manipulation: Add on empty cell click, 2D time/pitch drag, right-edge resize, right-click delete
//! - FL Studio-style Slide Notes with signature triangular glyphs
//! - Multi-channel Polyphonic Ghost Notes with calm-tech 28% opacity for harmonic cross-referencing
//! - Live Ghost Dragging with snap guidelines and floating HUD telemetry pill

use crate::fl_studio::piano_roll::{EventLaneTarget, MusicalScale};
use crate::ui::canvas_timeline::SnapGrid;
use crate::ui::message::UiMessage;
use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Size};

pub const KEYBOARD_WIDTH: f32 = 64.0;
pub const KEY_HEIGHT: f32 = 16.0;
pub const TOTAL_KEYS: usize = 128; // MIDI 0..127
pub const TICKS_PER_BEAT: u64 = 960;
pub const TICKS_PER_BAR: u64 = 3840;

/// Returns true if the MIDI key corresponds to a piano black key (C#, D#, F#, G#, A#)
#[inline]
pub fn is_black_key(key: u8) -> bool {
    matches!(key % 12, 1 | 3 | 6 | 8 | 10)
}

/// Converts a MIDI key number (0..127) to its musical name (e.g., 60 -> "C4", 61 -> "C#4")
pub fn midi_key_to_name(key: u8) -> String {
    let octave = (key as i32 / 12) - 1;
    let name = match key % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        _ => "",
    };
    format!("{}{}", name, octave)
}

/// Display model for an active Piano Roll note
#[derive(Debug, Clone, PartialEq)]
pub struct PianoRollNoteDisplay {
    pub id: usize,
    pub channel_id: usize,
    pub key: u8,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub velocity: f32,
    pub pan: f32,
    pub fine_pitch_cents: f32,
    pub is_slide: bool,
}

/// Display model for a background ghost note from inactive channels
#[derive(Debug, Clone, PartialEq)]
pub struct GhostNoteDisplay {
    pub key: u8,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub channel_name: String,
}

/// Dragging mode for active mouse interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PianoRollDragMode {
    MoveNote {
        orig_key: u8,
        orig_start: u64,
        offset_ticks: u64,
    },
    ResizeNote {
        orig_len: u64,
    },
}

/// Live preview telemetry during drag or resize
#[derive(Debug, Clone, PartialEq)]
pub struct PianoRollLiveDrag {
    pub note_id: usize,
    pub mode: PianoRollDragMode,
    pub target_key: u8,
    pub target_start_tick: u64,
    pub target_length_ticks: u64,
    pub delta_semitones: i8,
    pub delta_ticks: i64,
}

/// Interactive state maintained across frames
#[derive(Debug, Clone, Default)]
pub struct PianoRollInteractionState {
    pub active_drag: Option<(usize, PianoRollDragMode)>,
    pub live_preview: Option<PianoRollLiveDrag>,
    pub last_cursor_pos: Option<Point>,
}

/// Canvas Program for the 2D Graphical Piano Roll
pub struct PianoRollCanvasProgram {
    pub playhead_tick: u64,
    pub horizontal_zoom: f32,
    pub scroll_offset_y: f32,
    pub snap: SnapGrid,
    pub selected_channel_id: usize,
    pub notes: Vec<PianoRollNoteDisplay>,
    pub ghost_notes: Vec<GhostNoteDisplay>,
    pub is_fl_theme: bool,
    pub scale: MusicalScale,
    pub scale_root: u8,
    pub event_target: EventLaneTarget,
    pub event_lane_height: f32,
}

impl PianoRollCanvasProgram {
    pub fn new(playhead_tick: u64, horizontal_zoom: f32, scroll_offset_y: f32) -> Self {
        Self {
            playhead_tick,
            horizontal_zoom: horizontal_zoom.clamp(0.2, 8.0),
            scroll_offset_y: scroll_offset_y.max(0.0),
            snap: SnapGrid::Sixteenth,
            selected_channel_id: 0,
            notes: Vec::new(),
            ghost_notes: Vec::new(),
            is_fl_theme: false,
            scale: MusicalScale::default(),
            scale_root: 0,
            event_target: EventLaneTarget::Velocity,
            event_lane_height: 75.0,
        }
    }

    pub fn with_notes(mut self, notes: Vec<PianoRollNoteDisplay>) -> Self {
        self.notes = notes;
        self
    }

    pub fn with_ghost_notes(mut self, ghost_notes: Vec<GhostNoteDisplay>) -> Self {
        self.ghost_notes = ghost_notes;
        self
    }

    pub fn with_snap(mut self, snap: SnapGrid) -> Self {
        self.snap = snap;
        self
    }

    pub fn with_theme(mut self, is_fl_theme: bool) -> Self {
        self.is_fl_theme = is_fl_theme;
        self
    }

    pub fn with_channel_id(mut self, channel_id: usize) -> Self {
        self.selected_channel_id = channel_id;
        self
    }

    pub fn with_scale(mut self, scale: MusicalScale, root: u8) -> Self {
        self.scale = scale;
        self.scale_root = root;
        self
    }

    pub fn with_event_target(mut self, target: EventLaneTarget) -> Self {
        self.event_target = target;
        self
    }

    pub fn with_event_lane_height(mut self, height: f32) -> Self {
        self.event_lane_height = height.max(0.0);
        self
    }

    #[inline]
    pub fn tick_to_x(&self, tick: u64) -> f32 {
        KEYBOARD_WIDTH + (tick as f32 / TICKS_PER_BEAT as f32) * (80.0 * self.horizontal_zoom)
    }

    #[inline]
    pub fn x_to_tick(&self, x: f32) -> u64 {
        if x <= KEYBOARD_WIDTH || self.horizontal_zoom <= 0.0 {
            0
        } else {
            let px_from_origin = x - KEYBOARD_WIDTH;
            let beats = px_from_origin / (80.0 * self.horizontal_zoom);
            (beats * TICKS_PER_BEAT as f32).max(0.0) as u64
        }
    }

    #[inline]
    pub fn key_to_y(&self, key: u8) -> f32 {
        let inverted_key = (127 - (key as i32)).max(0) as f32;
        (inverted_key * KEY_HEIGHT) - self.scroll_offset_y
    }

    #[inline]
    pub fn y_to_key(&self, y: f32) -> u8 {
        let total_y = y + self.scroll_offset_y;
        if total_y < 0.0 {
            127
        } else {
            let idx = (total_y / KEY_HEIGHT) as i32;
            (127 - idx).clamp(0, 127) as u8
        }
    }

    #[inline]
    pub fn length_to_w(&self, length_ticks: u64) -> f32 {
        (length_ticks as f32 / TICKS_PER_BEAT as f32) * (80.0 * self.horizontal_zoom)
    }
}

impl<Theme> canvas::Program<UiMessage, Theme> for PianoRollCanvasProgram {
    type State = PianoRollInteractionState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<UiMessage>> {
        let cursor_pos = cursor.position_in(bounds)?;
        state.last_cursor_pos = Some(cursor_pos);

        match event {
            // Left Mouse Button Pressed
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // 1. Click on Left Keyboard Sidebar
                if cursor_pos.x < KEYBOARD_WIDTH {
                    let clicked_key = self.y_to_key(cursor_pos.y);
                    return Some(Action::publish(UiMessage::PianoRollSelectKey(clicked_key)));
                }

                // 1b. Click in Lower Event Editor Lane
                let lane_y = bounds.height - self.event_lane_height;
                if cursor_pos.y >= lane_y && self.event_lane_height > 20.0 {
                    if let Some(closest) = self.notes.iter().min_by_key(|n| {
                        let nx = self.tick_to_x(n.start_tick);
                        ((nx - cursor_pos.x).abs() * 100.0) as i64
                    }) {
                        let usable_h = (self.event_lane_height - 16.0).max(1.0);
                        let val = match self.event_target {
                            EventLaneTarget::Velocity => {
                                let frac = 1.0 - (cursor_pos.y - (lane_y + 4.0)) / usable_h;
                                frac.clamp(0.05, 1.0)
                            }
                            EventLaneTarget::Pan => {
                                let mid_y = lane_y + (self.event_lane_height * 0.5);
                                let frac = (mid_y - cursor_pos.y) / (usable_h * 0.5);
                                frac.clamp(-1.0, 1.0)
                            }
                            EventLaneTarget::Pitch => {
                                let mid_y = lane_y + (self.event_lane_height * 0.5);
                                let frac = (mid_y - cursor_pos.y) / (usable_h * 0.5);
                                (frac * 100.0).clamp(-100.0, 100.0)
                            }
                        };
                        return Some(Action::publish(UiMessage::PianoRollSetNoteProperty {
                            note_id: closest.id,
                            property: self.event_target,
                            value: val,
                        }));
                    }
                    return Some(Action::request_redraw());
                }

                // 2. Click in 2D Note Grid: Hit-test existing active notes (reverse order)
                for note in self.notes.iter().rev() {
                    let nx = self.tick_to_x(note.start_tick);
                    let nw = self.length_to_w(note.length_ticks).max(8.0);
                    let ny = self.key_to_y(note.key);

                    if cursor_pos.x >= nx
                        && cursor_pos.x <= nx + nw
                        && cursor_pos.y >= ny
                        && cursor_pos.y <= ny + KEY_HEIGHT
                    {
                        // Right edge resize handle (within 8px of note end)
                        if cursor_pos.x >= (nx + nw - 8.0) {
                            let mode = PianoRollDragMode::ResizeNote {
                                orig_len: note.length_ticks,
                            };
                            state.active_drag = Some((note.id, mode));
                            state.live_preview = Some(PianoRollLiveDrag {
                                note_id: note.id,
                                mode,
                                target_key: note.key,
                                target_start_tick: note.start_tick,
                                target_length_ticks: note.length_ticks,
                                delta_semitones: 0,
                                delta_ticks: 0,
                            });
                            return Some(Action::request_redraw());
                        }

                        // Note Body Drag
                        let cursor_tick = self.x_to_tick(cursor_pos.x);
                        let offset_ticks = cursor_tick.saturating_sub(note.start_tick);
                        let mode = PianoRollDragMode::MoveNote {
                            orig_key: note.key,
                            orig_start: note.start_tick,
                            offset_ticks,
                        };
                        state.active_drag = Some((note.id, mode));
                        state.live_preview = Some(PianoRollLiveDrag {
                            note_id: note.id,
                            mode,
                            target_key: note.key,
                            target_start_tick: note.start_tick,
                            target_length_ticks: note.length_ticks,
                            delta_semitones: 0,
                            delta_ticks: 0,
                        });
                        return Some(Action::request_redraw());
                    }
                }

                // 3. Click on Empty Grid: Spawn a new note at snapped tick & key
                let cursor_tick = self.x_to_tick(cursor_pos.x);
                let snapped_start = self.snap.snap(cursor_tick, self.horizontal_zoom);
                let target_key = self.y_to_key(cursor_pos.y);
                let snap_step = self.snap.snap_step_ticks(self.horizontal_zoom).max(240);

                Some(Action::publish(UiMessage::PianoRollAddNote {
                    channel_id: self.selected_channel_id,
                    key: target_key,
                    start_tick: snapped_start,
                    length_ticks: snap_step,
                }))
            }

            // Right Mouse Button Pressed: Delete Note
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if cursor_pos.x >= KEYBOARD_WIDTH {
                    for note in self.notes.iter().rev() {
                        let nx = self.tick_to_x(note.start_tick);
                        let nw = self.length_to_w(note.length_ticks).max(8.0);
                        let ny = self.key_to_y(note.key);

                        if cursor_pos.x >= nx
                            && cursor_pos.x <= nx + nw
                            && cursor_pos.y >= ny
                            && cursor_pos.y <= ny + KEY_HEIGHT
                        {
                            return Some(Action::publish(UiMessage::PianoRollDeleteNote(note.id)));
                        }
                    }
                }
                None
            }

            // Middle Mouse Button Pressed: Toggle Slide state on clicked note
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                if cursor_pos.x >= KEYBOARD_WIDTH {
                    for note in self.notes.iter().rev() {
                        let nx = self.tick_to_x(note.start_tick);
                        let nw = self.length_to_w(note.length_ticks).max(8.0);
                        let ny = self.key_to_y(note.key);

                        if cursor_pos.x >= nx
                            && cursor_pos.x <= nx + nw
                            && cursor_pos.y >= ny
                            && cursor_pos.y <= ny + KEY_HEIGHT
                        {
                            return Some(Action::publish(UiMessage::PianoRollToggleSlide(note.id)));
                        }
                    }
                }
                None
            }

            // Cursor Moved: Update live preview during drag or resize
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some((note_id, mode)) = state.active_drag
                    && let Some(note) = self.notes.iter().find(|n| n.id == note_id)
                {
                    match mode {
                        PianoRollDragMode::MoveNote {
                            orig_key,
                            orig_start,
                            offset_ticks,
                        } => {
                            let cand_key = self.y_to_key(cursor_pos.y);
                            let cand_tick_raw = self.x_to_tick(cursor_pos.x);
                            let cand_start_raw =
                                (cand_tick_raw as i64 - offset_ticks as i64).max(0) as u64;
                            let cand_start = self.snap.snap(cand_start_raw, self.horizontal_zoom);
                            let delta_semitones = cand_key as i8 - orig_key as i8;
                            let delta_ticks = cand_start as i64 - orig_start as i64;

                            state.live_preview = Some(PianoRollLiveDrag {
                                note_id,
                                mode,
                                target_key: cand_key,
                                target_start_tick: cand_start,
                                target_length_ticks: note.length_ticks,
                                delta_semitones,
                                delta_ticks,
                            });
                            return Some(Action::request_redraw());
                        }
                        PianoRollDragMode::ResizeNote { orig_len } => {
                            let cand_end_raw = self.x_to_tick(cursor_pos.x);
                            let cand_end = self.snap.snap(cand_end_raw, self.horizontal_zoom);
                            let cand_len = cand_end.saturating_sub(note.start_tick).max(60);
                            let delta_ticks = cand_len as i64 - orig_len as i64;

                            state.live_preview = Some(PianoRollLiveDrag {
                                note_id,
                                mode,
                                target_key: note.key,
                                target_start_tick: note.start_tick,
                                target_length_ticks: cand_len,
                                delta_semitones: 0,
                                delta_ticks,
                            });
                            return Some(Action::request_redraw());
                        }
                    }
                }
                None
            }

            // Left Mouse Button Released: Commit move or resize
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(preview) = state.live_preview.take() {
                    state.active_drag = None;
                    match preview.mode {
                        PianoRollDragMode::MoveNote { .. } => {
                            if preview.delta_ticks != 0 || preview.delta_semitones != 0 {
                                return Some(Action::publish(UiMessage::PianoRollMoveNote {
                                    id: preview.note_id,
                                    delta_ticks: preview.delta_ticks,
                                    delta_key: preview.delta_semitones,
                                }));
                            }
                        }
                        PianoRollDragMode::ResizeNote { .. } => {
                            if preview.delta_ticks != 0 {
                                return Some(Action::publish(UiMessage::PianoRollResizeNote {
                                    id: preview.note_id,
                                    delta_ticks: preview.delta_ticks,
                                }));
                            }
                        }
                    }
                    return Some(Action::request_redraw());
                }
                None
            }

            // Mouse Wheel: Vertical scrolling through 128 keys
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 24.0,
                    mouse::ScrollDelta::Pixels { y, .. } => *y,
                };
                let max_scroll = ((TOTAL_KEYS as f32 * KEY_HEIGHT) - bounds.height).max(0.0);
                let new_scroll = (self.scroll_offset_y - dy).clamp(0.0, max_scroll);
                Some(Action::publish(UiMessage::PianoRollSetScrollY(new_scroll)))
            }

            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // 1. Overall Dark Viewport Background
        let base_bg = if self.is_fl_theme {
            Color::from_rgb8(0x18, 0x1a, 0x1e)
        } else {
            Color::from_rgb8(0x14, 0x16, 0x1b)
        };
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), base_bg);

        // 2. Horizontal Key Row Stripes (Checkered FL / Logic style)
        for key in 0..128u8 {
            let y = self.key_to_y(key);
            if y + KEY_HEIGHT < 0.0 || y > bounds.height {
                continue;
            }

            let is_black = is_black_key(key);
            let in_scale = self.scale.is_in_scale(self.scale_root, key);
            let row_bg = if !in_scale {
                Color::from_rgba8(0x05, 0x07, 0x0a, 0.65)
            } else if is_black {
                Color::from_rgba8(0x0a, 0x0c, 0x10, 0.40)
            } else {
                Color::from_rgba8(0xff, 0xff, 0xff, 0.02)
            };
            frame.fill_rectangle(
                Point::new(KEYBOARD_WIDTH, y),
                Size::new(bounds.width - KEYBOARD_WIDTH, KEY_HEIGHT),
                row_bg,
            );

            // Subtle row divider line
            let divider_color = if key % 12 == 0 {
                // Octave C line: slightly brighter
                Color::from_rgba8(0xff, 0xff, 0xff, 0.12)
            } else {
                Color::from_rgba8(0xff, 0xff, 0xff, 0.03)
            };
            let row_line = Path::line(
                Point::new(KEYBOARD_WIDTH, y + KEY_HEIGHT),
                Point::new(bounds.width, y + KEY_HEIGHT),
            );
            frame.stroke(
                &row_line,
                Stroke::default().with_color(divider_color).with_width(1.0),
            );
        }

        // 3. Vertical Time Grid: Sub-beats, Beats, and Bars
        let start_tick = self.x_to_tick(KEYBOARD_WIDTH);
        let end_tick = self.x_to_tick(bounds.width);
        let start_bar = start_tick / TICKS_PER_BAR;
        let end_bar = (end_tick / TICKS_PER_BAR) + 1;

        for bar in start_bar..=end_bar {
            let bar_tick = bar * TICKS_PER_BAR;
            let bar_x = self.tick_to_x(bar_tick);

            // 3a. Subdivisions inside each bar (beats and 16ths)
            for beat in 1..4 {
                let beat_tick = bar_tick + (beat * TICKS_PER_BEAT);
                let beat_x = self.tick_to_x(beat_tick);
                if beat_x >= KEYBOARD_WIDTH && beat_x <= bounds.width {
                    let beat_line =
                        Path::line(Point::new(beat_x, 0.0), Point::new(beat_x, bounds.height));
                    frame.stroke(
                        &beat_line,
                        Stroke::default()
                            .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.07))
                            .with_width(1.0),
                    );
                }
            }

            // 3b. Solid Bar Boundary Line
            if bar_x >= KEYBOARD_WIDTH && bar_x <= bounds.width {
                let bar_line = Path::line(Point::new(bar_x, 0.0), Point::new(bar_x, bounds.height));
                frame.stroke(
                    &bar_line,
                    Stroke::default()
                        .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.18))
                        .with_width(1.5),
                );

                // Small Bar number at the very top of each bar line
                frame.fill_text(canvas::Text {
                    content: format!("{}", bar + 1),
                    position: Point::new(bar_x + 4.0, 2.0),
                    color: Color::from_rgba8(0xff, 0xff, 0xff, 0.35),
                    size: 9.0.into(),
                    ..Default::default()
                });
            }
        }

        // 4. Ghost Notes from Background Channels (Calm-tech 28% opacity)
        for ghost in &self.ghost_notes {
            let gx = self.tick_to_x(ghost.start_tick);
            let gw = self.length_to_w(ghost.length_ticks).max(4.0);
            let gy = self.key_to_y(ghost.key);

            if gx + gw < KEYBOARD_WIDTH
                || gx > bounds.width
                || gy + KEY_HEIGHT < 0.0
                || gy > bounds.height
            {
                continue;
            }

            let ghost_rect = Rectangle::new(
                Point::new(gx.max(KEYBOARD_WIDTH), gy + 1.0),
                Size::new(gw, KEY_HEIGHT - 2.0),
            );
            frame.fill_rectangle(
                ghost_rect.position(),
                ghost_rect.size(),
                Color::from_rgba8(0x9a, 0xb0, 0xcc, 0.24),
            );
            frame.stroke(
                &Path::rectangle(ghost_rect.position(), ghost_rect.size()),
                Stroke::default()
                    .with_color(Color::from_rgba8(0xbb, 0xd0, 0xee, 0.35))
                    .with_width(1.0),
            );

            // Channel name tag if note is wide enough
            if gw > 40.0 {
                frame.fill_text(canvas::Text {
                    content: ghost.channel_name.clone(),
                    position: Point::new(ghost_rect.x + 3.0, ghost_rect.y + 2.0),
                    color: Color::from_rgba8(0xff, 0xff, 0xff, 0.40),
                    size: 9.0.into(),
                    ..Default::default()
                });
            }
        }

        // 5. Active Piano Roll Notes
        let note_primary_color = if self.is_fl_theme {
            Color::from_rgb8(0xfa, 0x82, 0x31) // FL Studio vibrant orange
        } else {
            Color::from_rgb8(0x2e, 0xcc, 0x71) // Logic / Modern DAW emerald green
        };

        for note in &self.notes {
            // If currently dragging this note, render original in dim ghost state
            let is_being_dragged = state
                .live_preview
                .as_ref()
                .is_some_and(|p| p.note_id == note.id);

            let nx = self.tick_to_x(note.start_tick);
            let nw = self.length_to_w(note.length_ticks).max(4.0);
            let ny = self.key_to_y(note.key);

            if nx + nw < KEYBOARD_WIDTH
                || nx > bounds.width
                || ny + KEY_HEIGHT < 0.0
                || ny > bounds.height
            {
                continue;
            }

            let fill_color = if is_being_dragged {
                Color::from_rgba(
                    note_primary_color.r,
                    note_primary_color.g,
                    note_primary_color.b,
                    0.30,
                )
            } else {
                // Modulate brightness slightly by velocity
                let vel_factor = (note.velocity * 0.4) + 0.6;
                Color::from_rgb(
                    (note_primary_color.r * vel_factor).min(1.0),
                    (note_primary_color.g * vel_factor).min(1.0),
                    (note_primary_color.b * vel_factor).min(1.0),
                )
            };

            let note_rect = Rectangle::new(
                Point::new(nx.max(KEYBOARD_WIDTH), ny + 1.0),
                Size::new(nw, KEY_HEIGHT - 2.0),
            );

            frame.fill_rectangle(note_rect.position(), note_rect.size(), fill_color);

            // Subtle border outline & bevel
            frame.stroke(
                &Path::rectangle(note_rect.position(), note_rect.size()),
                Stroke::default()
                    .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.40))
                    .with_width(1.0),
            );

            // Resize handle notch on right edge
            let notch_x = note_rect.x + note_rect.width - 2.0;
            if notch_x > note_rect.x + 6.0 {
                let notch_line = Path::line(
                    Point::new(notch_x, note_rect.y + 2.0),
                    Point::new(notch_x, note_rect.y + note_rect.height - 2.0),
                );
                frame.stroke(
                    &notch_line,
                    Stroke::default()
                        .with_color(Color::from_rgba8(0x00, 0x00, 0x00, 0.40))
                        .with_width(1.5),
                );
            }

            // FL Studio Slide Triangle Glyph on top-left of note
            if note.is_slide {
                let tri_path = Path::new(|builder| {
                    builder.move_to(Point::new(note_rect.x + 2.0, note_rect.y + 2.0));
                    builder.line_to(Point::new(note_rect.x + 9.0, note_rect.y + 2.0));
                    builder.line_to(Point::new(note_rect.x + 2.0, note_rect.y + 9.0));
                    builder.close();
                });
                frame.fill(&tri_path, Color::WHITE);
            }

            // Note Name label inside body if wide enough
            if nw > 28.0 {
                let key_name = midi_key_to_name(note.key);
                let slide_flag = if note.is_slide { " [S]" } else { "" };
                frame.fill_text(canvas::Text {
                    content: format!("{}{}", key_name, slide_flag),
                    position: Point::new(note_rect.x + 4.0, note_rect.y + 2.0),
                    color: Color::from_rgb8(0x10, 0x15, 0x20),
                    size: 9.5.into(),
                    ..Default::default()
                });
            }
        }

        // 6. Live Ghost Dragging / Resizing Preview Box
        if let Some(preview) = &state.live_preview {
            let px = self.tick_to_x(preview.target_start_tick);
            let pw = self.length_to_w(preview.target_length_ticks).max(4.0);
            let py = self.key_to_y(preview.target_key);

            // Candidate destination ghost box (50% cyan)
            let ghost_color = Color::from_rgba8(0x00, 0xe5, 0xff, 0.50);
            let ghost_rect = Rectangle::new(
                Point::new(px.max(KEYBOARD_WIDTH), py + 1.0),
                Size::new(pw, KEY_HEIGHT - 2.0),
            );
            frame.fill_rectangle(ghost_rect.position(), ghost_rect.size(), ghost_color);
            frame.stroke(
                &Path::rectangle(ghost_rect.position(), ghost_rect.size()),
                Stroke::default().with_color(Color::WHITE).with_width(1.5),
            );

            // Vertical Snap Guideline across the entire canvas height
            let snap_line = Path::line(Point::new(px, 0.0), Point::new(px, bounds.height));
            frame.stroke(
                &snap_line,
                Stroke::default()
                    .with_color(Color::from_rgba8(0x00, 0xe5, 0xff, 0.70))
                    .with_width(1.5),
            );

            // Floating HUD Telemetry Pill
            let key_str = midi_key_to_name(preview.target_key);
            let delta_key_str = if preview.delta_semitones != 0 {
                format!(" ({:+})", preview.delta_semitones)
            } else {
                String::new()
            };
            let delta_tick_str = if preview.delta_ticks != 0 {
                format!(" {:+} ticks", preview.delta_ticks)
            } else {
                String::new()
            };

            let hud_text = format!(
                "{} | Bar {}.{} |{}{}",
                key_str,
                (preview.target_start_tick / TICKS_PER_BAR) + 1,
                ((preview.target_start_tick % TICKS_PER_BAR) / TICKS_PER_BEAT) + 1,
                delta_key_str,
                delta_tick_str
            );

            let hud_x = px.clamp(KEYBOARD_WIDTH + 10.0, bounds.width - 180.0);
            let hud_y = (py - 22.0).max(10.0);
            let pill_rect = Rectangle::new(Point::new(hud_x, hud_y), Size::new(175.0, 18.0));
            frame.fill_rectangle(
                pill_rect.position(),
                pill_rect.size(),
                Color::from_rgba8(0x10, 0x15, 0x22, 0.90),
            );
            frame.stroke(
                &Path::rectangle(pill_rect.position(), pill_rect.size()),
                Stroke::default()
                    .with_color(Color::from_rgba8(0x00, 0xe5, 0xff, 0.75))
                    .with_width(1.0),
            );
            frame.fill_text(canvas::Text {
                content: hud_text,
                position: Point::new(pill_rect.x + 4.0, pill_rect.y + 3.0),
                color: Color::from_rgb8(0x00, 0xe5, 0xff),
                size: 9.5.into(),
                ..Default::default()
            });
        }

        // 7. Playhead Laser Needle
        let playhead_x = self.tick_to_x(self.playhead_tick);
        if playhead_x >= KEYBOARD_WIDTH && playhead_x <= bounds.width {
            let ph_line = Path::line(
                Point::new(playhead_x, 0.0),
                Point::new(playhead_x, bounds.height),
            );
            // Outer glow
            frame.stroke(
                &ph_line,
                Stroke::default()
                    .with_color(Color::from_rgba8(0xff, 0x9f, 0x43, 0.40))
                    .with_width(3.0),
            );
            // Core white needle
            frame.stroke(
                &ph_line,
                Stroke::default().with_color(Color::WHITE).with_width(1.2),
            );
        }

        // 8. Vertical Keyboard Sidebar (The 128-Key Rake on the left)
        let kb_bg = Color::from_rgb8(0x1c, 0x1e, 0x24);
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(KEYBOARD_WIDTH, bounds.height),
            kb_bg,
        );

        for key in 0..128u8 {
            let y = self.key_to_y(key);
            if y + KEY_HEIGHT < 0.0 || y > bounds.height {
                continue;
            }

            let is_black = is_black_key(key);
            if is_black {
                // Black Piano Key (narrower width 60%)
                let black_w = KEYBOARD_WIDTH * 0.62;
                frame.fill_rectangle(
                    Point::new(0.0, y),
                    Size::new(black_w, KEY_HEIGHT),
                    Color::from_rgb8(0x22, 0x24, 0x2b),
                );
                frame.stroke(
                    &Path::rectangle(Point::new(0.0, y), Size::new(black_w, KEY_HEIGHT)),
                    Stroke::default()
                        .with_color(Color::from_rgb8(0x11, 0x12, 0x16))
                        .with_width(1.0),
                );
            } else {
                // White Piano Key (full width)
                frame.fill_rectangle(
                    Point::new(0.0, y),
                    Size::new(KEYBOARD_WIDTH, KEY_HEIGHT),
                    Color::from_rgb8(0xeb, 0xee, 0xf2),
                );
                frame.stroke(
                    &Path::rectangle(Point::new(0.0, y), Size::new(KEYBOARD_WIDTH, KEY_HEIGHT)),
                    Stroke::default()
                        .with_color(Color::from_rgb8(0xba, 0xbe, 0xc6))
                        .with_width(1.0),
                );

                // High-contrast C note marker with octave number (e.g. C4, C5)
                if key % 12 == 0 {
                    let note_str = midi_key_to_name(key);
                    frame.fill_text(canvas::Text {
                        content: note_str,
                        position: Point::new(KEYBOARD_WIDTH - 24.0, y + 2.5),
                        color: Color::from_rgb8(0x18, 0x1a, 0x20),
                        size: 9.5.into(),
                        ..Default::default()
                    });
                }
            }
        }

        // Right shadow line separating keyboard from grid
        let sep_line = Path::line(
            Point::new(KEYBOARD_WIDTH, 0.0),
            Point::new(KEYBOARD_WIDTH, bounds.height),
        );
        frame.stroke(
            &sep_line,
            Stroke::default()
                .with_color(Color::from_rgba8(0x00, 0x00, 0x00, 0.60))
                .with_width(2.0),
        );

        // 9. Lower Event Editor Lane (Velocity / Pan / Pitch)
        let lane_h = self.event_lane_height;
        let lane_y = bounds.height - lane_h;
        if lane_h > 20.0 && lane_y > 0.0 {
            // Background well
            frame.fill_rectangle(
                Point::new(0.0, lane_y),
                Size::new(bounds.width, lane_h),
                Color::from_rgb8(0x10, 0x12, 0x17),
            );
            // Top border divider
            let lane_sep = Path::line(Point::new(0.0, lane_y), Point::new(bounds.width, lane_y));
            frame.stroke(
                &lane_sep,
                Stroke::default()
                    .with_color(Color::from_rgb8(0x2d, 0x33, 0x42))
                    .with_width(1.5),
            );

            // Left header label in the keyboard column
            frame.fill_rectangle(
                Point::new(0.0, lane_y),
                Size::new(KEYBOARD_WIDTH, lane_h),
                Color::from_rgb8(0x16, 0x18, 0x20),
            );
            let target_str = match self.event_target {
                EventLaneTarget::Velocity => "VELOCITY",
                EventLaneTarget::Pan => "PAN",
                EventLaneTarget::Pitch => "PITCH",
            };
            frame.fill_text(canvas::Text {
                content: target_str.to_string(),
                position: Point::new(4.0, lane_y + 6.0),
                color: Color::from_rgb8(0xfa, 0x82, 0x31),
                size: 8.5.into(),
                ..Default::default()
            });

            // Center baseline for bi-polar parameters (Pan, Pitch)
            if self.event_target == EventLaneTarget::Pan
                || self.event_target == EventLaneTarget::Pitch
            {
                let mid_y = lane_y + (lane_h * 0.5);
                let mid_line = Path::line(
                    Point::new(KEYBOARD_WIDTH, mid_y),
                    Point::new(bounds.width, mid_y),
                );
                frame.stroke(
                    &mid_line,
                    Stroke::default()
                        .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.10))
                        .with_width(1.0),
                );
            }

            // Draw note stems and node heads
            let stem_usable_h = lane_h - 16.0;
            for note in &self.notes {
                let nx = self.tick_to_x(note.start_tick);
                if nx < KEYBOARD_WIDTH || nx > bounds.width {
                    continue;
                }

                let (stem_y_top, stem_y_bottom, node_val_str) = match self.event_target {
                    EventLaneTarget::Velocity => {
                        let h = (note.velocity * stem_usable_h).clamp(2.0, stem_usable_h);
                        (
                            lane_y + lane_h - h - 4.0,
                            lane_y + lane_h - 4.0,
                            format!("{:.0}%", note.velocity * 100.0),
                        )
                    }
                    EventLaneTarget::Pan => {
                        let mid_y = lane_y + (lane_h * 0.5);
                        let offset = (note.pan * (stem_usable_h * 0.5))
                            .clamp(-stem_usable_h * 0.5, stem_usable_h * 0.5);
                        let top = mid_y - offset;
                        (
                            top,
                            mid_y,
                            if note.pan.abs() < 0.05 {
                                "C".to_string()
                            } else if note.pan < 0.0 {
                                format!("L{:.0}", -note.pan * 100.0)
                            } else {
                                format!("R{:.0}", note.pan * 100.0)
                            },
                        )
                    }
                    EventLaneTarget::Pitch => {
                        let mid_y = lane_y + (lane_h * 0.5);
                        let offset = ((note.fine_pitch_cents / 100.0) * (stem_usable_h * 0.5))
                            .clamp(-stem_usable_h * 0.5, stem_usable_h * 0.5);
                        let top = mid_y - offset;
                        (top, mid_y, format!("{:+.0}c", note.fine_pitch_cents))
                    }
                };

                // Vertical stem line
                let stem_line =
                    Path::line(Point::new(nx, stem_y_bottom), Point::new(nx, stem_y_top));
                frame.stroke(
                    &stem_line,
                    Stroke::default()
                        .with_color(Color::from_rgba8(0xfa, 0x82, 0x31, 0.75))
                        .with_width(1.5),
                );

                // Circular node cap at the top of the stem
                let cap_circle = Path::circle(Point::new(nx, stem_y_top), 3.0);
                frame.fill(&cap_circle, Color::from_rgb8(0xfa, 0x82, 0x31));

                // Value readout if room
                frame.fill_text(canvas::Text {
                    content: node_val_str,
                    position: Point::new(nx + 3.0, (stem_y_top - 6.0).max(lane_y + 2.0)),
                    color: Color::from_rgba8(0xff, 0xff, 0xff, 0.50),
                    size: 8.0.into(),
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }
}

/// Helper function to create the Piano Roll Canvas Element
pub fn view_canvas_piano_roll<'a>(program: PianoRollCanvasProgram) -> Element<'a, UiMessage> {
    Canvas::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
