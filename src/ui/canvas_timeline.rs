use crate::ui::message::UiMessage;
use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Size};

pub const RULER_HEIGHT: f32 = 30.0;
pub const LANE_HEIGHT: f32 = 80.0;
pub const PPQN: u64 = 960;
pub const TICKS_PER_BAR: u64 = 3840; // 4/4 time

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartToolZone {
    TrimStart,
    TrimEnd,
    FadeIn,
    FadeOut,
    GrabMove,
    SelectRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleDragMode {
    MoveEntire,
    AdjustStart,
    AdjustEnd,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveformPeak {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone)]
pub enum ClipVisualKind {
    Audio {
        waveform_peaks: Vec<(f32, f32)>, // (min, max) normalized -1.0..1.0
    },
    Midi {
        notes: Vec<(u64, u8, u64)>, // (rel_start_tick, pitch 0..127, length_ticks)
    },
    Pattern,
}

#[derive(Debug, Clone)]
pub struct TimelineClipDisplay {
    pub id: usize,
    pub track_index: usize,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub name: String,
    pub color_rgb: [u8; 3],
    pub muted: bool,
    pub looped: bool,
    pub fade_in_ticks: u64,
    pub fade_out_ticks: u64,
    pub kind: ClipVisualKind,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum SnapGrid {
    Off,
    Bar,
    Half,
    Beat,
    Eighth,
    #[default]
    Sixteenth,
    ThirtySecond,
    Smart,
}

impl SnapGrid {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Off => "Snap: Off",
            Self::Bar => "Snap: 1 Bar",
            Self::Half => "Snap: 1/2",
            Self::Beat => "Snap: 1/4 (Beat)",
            Self::Eighth => "Snap: 1/8",
            Self::Sixteenth => "Snap: 1/16",
            Self::ThirtySecond => "Snap: 1/32",
            Self::Smart => "Snap: Smart",
        }
    }

    pub fn snap_step_ticks(&self, zoom: f32) -> u64 {
        match self {
            Self::Off => 1,
            Self::Bar => TICKS_PER_BAR,
            Self::Half => TICKS_PER_BAR / 2,
            Self::Beat => PPQN,
            Self::Eighth => PPQN / 2,
            Self::Sixteenth => PPQN / 4,
            Self::ThirtySecond => PPQN / 8,
            Self::Smart => {
                if zoom > 4.0 {
                    PPQN / 8
                } else if zoom > 1.8 {
                    PPQN / 4
                } else if zoom > 0.8 {
                    PPQN / 2
                } else {
                    PPQN
                }
            }
        }
    }

    pub fn snap(&self, tick: u64, zoom: f32) -> u64 {
        let step = self.snap_step_ticks(zoom);
        if step <= 1 {
            tick
        } else {
            let rem = tick % step;
            if rem >= step / 2 {
                tick - rem + step
            } else {
                tick - rem
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CanvasInteractionState {
    pub is_scrubbing_playhead: bool,
    pub cycle_drag: Option<(CycleDragMode, u64, u64)>, // mode, start_anchor, end_anchor
    pub dragged_clip: Option<(usize, SmartToolZone, usize, u64, u64, u64)>, // clip_id, zone, orig_track, orig_start, orig_len, offset_ticks
    pub marquee_drag: Option<(Point, Point)>, // start_anchor, current_anchor
    pub last_cursor_pos: Option<Point>,
}

pub struct ArrangementCanvasProgram {
    pub playhead_tick: u64,
    pub visible_start_tick: u64,
    pub visible_end_tick: u64,
    pub horizontal_zoom: f32,
    pub track_count: usize,
    pub loop_range: Option<(u64, u64)>,
    pub clips: Vec<TimelineClipDisplay>,
    pub selected_track_id: usize,
    pub snap: SnapGrid,
    pub scroll_offset_y: f32,
    pub marquee_selection: Option<(u64, u64, usize, usize)>,
    pub is_fl_theme: bool,
}

impl ArrangementCanvasProgram {
    pub fn new(playhead_tick: u64, horizontal_zoom: f32, track_count: usize) -> Self {
        Self {
            playhead_tick,
            visible_start_tick: 0,
            visible_end_tick: TICKS_PER_BAR * 32,
            horizontal_zoom: horizontal_zoom.clamp(0.2, 10.0),
            track_count: track_count.max(1),
            loop_range: Some((0, TICKS_PER_BAR * 4)),
            clips: Vec::new(),
            selected_track_id: 0,
            snap: SnapGrid::Sixteenth,
            scroll_offset_y: 0.0,
            marquee_selection: None,
            is_fl_theme: false,
        }
    }

    pub fn with_theme(mut self, is_fl_theme: bool) -> Self {
        self.is_fl_theme = is_fl_theme;
        self
    }

    pub fn with_snap(mut self, snap: SnapGrid) -> Self {
        self.snap = snap;
        self
    }

    pub fn with_scroll_y(mut self, scroll_offset_y: f32) -> Self {
        self.scroll_offset_y = scroll_offset_y.max(0.0);
        self
    }

    pub fn with_marquee(mut self, marquee: Option<(u64, u64, usize, usize)>) -> Self {
        self.marquee_selection = marquee;
        self
    }

    pub fn tick_to_x(&self, tick: u64, bounds_width: f32) -> f32 {
        let total_ticks = self
            .visible_end_tick
            .saturating_sub(self.visible_start_tick)
            .max(1);
        let progress = (tick.saturating_sub(self.visible_start_tick) as f32) / (total_ticks as f32);
        (progress * bounds_width * self.horizontal_zoom).clamp(0.0, bounds_width * 100.0)
    }

    pub fn x_to_tick(&self, x: f32, bounds_width: f32) -> u64 {
        if bounds_width <= 0.0 || self.horizontal_zoom <= 0.0 {
            return 0;
        }
        let total_ticks = self
            .visible_end_tick
            .saturating_sub(self.visible_start_tick)
            .max(1);
        let progress = (x / (bounds_width * self.horizontal_zoom)).clamp(0.0, 1.0);
        self.visible_start_tick + (progress * total_ticks as f32) as u64
    }

    pub fn y_to_track(&self, y: f32) -> Option<usize> {
        if y < RULER_HEIGHT {
            None
        } else {
            let scrolled_y = y - RULER_HEIGHT + self.scroll_offset_y;
            if scrolled_y < 0.0 {
                return None;
            }
            let trk = (scrolled_y / LANE_HEIGHT) as usize;
            Some(trk.min(self.track_count.saturating_sub(1)))
        }
    }

    pub fn track_to_y(&self, track_index: usize) -> f32 {
        RULER_HEIGHT + (track_index as f32 * LANE_HEIGHT) - self.scroll_offset_y
    }
}

impl<Theme> canvas::Program<UiMessage, Theme> for ArrangementCanvasProgram {
    type State = CanvasInteractionState;

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
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let target_tick = self.x_to_tick(cursor_pos.x, bounds.width);

                // 1. Click in Top Ruler
                if cursor_pos.y < RULER_HEIGHT {
                    if cursor_pos.y < 14.0 {
                        // Top Cycle Strip interaction
                        if let Some((start_tick, end_tick)) = self.loop_range {
                            let start_x = self.tick_to_x(start_tick, bounds.width);
                            let end_x = self.tick_to_x(end_tick, bounds.width);

                            if (cursor_pos.x - start_x).abs() <= 8.0 {
                                state.cycle_drag =
                                    Some((CycleDragMode::AdjustStart, start_tick, end_tick));
                                return None;
                            } else if (cursor_pos.x - end_x).abs() <= 8.0 {
                                state.cycle_drag =
                                    Some((CycleDragMode::AdjustEnd, start_tick, end_tick));
                                return None;
                            } else if cursor_pos.x > start_x && cursor_pos.x < end_x {
                                state.cycle_drag =
                                    Some((CycleDragMode::MoveEntire, start_tick, end_tick));
                                return None;
                            }
                        }
                        // Start new cycle selection snapped to bar
                        let snapped = (target_tick / TICKS_PER_BAR) * TICKS_PER_BAR;
                        state.cycle_drag =
                            Some((CycleDragMode::AdjustEnd, snapped, snapped + TICKS_PER_BAR));
                        return Some(Action::publish(UiMessage::SetCycleRange {
                            start_tick: snapped,
                            end_tick: snapped + TICKS_PER_BAR,
                        }));
                    } else {
                        // Bottom ruler tick area: scrub playhead
                        state.is_scrubbing_playhead = true;
                        return Some(Action::publish(UiMessage::SetPlayhead(target_tick)));
                    }
                }

                // 2. Click in Track Lanes
                if let Some(target_trk) = self.y_to_track(cursor_pos.y) {
                    let clicked_clip = self.clips.iter().find(|clip| {
                        clip.track_index == target_trk
                            && target_tick >= clip.start_tick
                            && target_tick < (clip.start_tick + clip.length_ticks)
                    });

                    if let Some(clip) = clicked_clip {
                        let x_start = self.tick_to_x(clip.start_tick, bounds.width);
                        let x_end =
                            self.tick_to_x(clip.start_tick + clip.length_ticks, bounds.width);
                        let y_clip = RULER_HEIGHT + (target_trk as f32 * LANE_HEIGHT) + 4.0;
                        let h_clip = LANE_HEIGHT - 8.0;

                        // Smart Tool Zone Resolution
                        let zone = if cursor_pos.x - x_start < 8.0 {
                            SmartToolZone::TrimStart
                        } else if x_end - cursor_pos.x < 8.0 {
                            SmartToolZone::TrimEnd
                        } else if cursor_pos.y - y_clip < 14.0 && cursor_pos.x - x_start < 16.0 {
                            SmartToolZone::FadeIn
                        } else if cursor_pos.y - y_clip < 14.0 && x_end - cursor_pos.x < 16.0 {
                            SmartToolZone::FadeOut
                        } else if cursor_pos.y > y_clip + (h_clip * 0.45) {
                            SmartToolZone::GrabMove
                        } else {
                            SmartToolZone::SelectRange
                        };

                        let offset_ticks = target_tick.saturating_sub(clip.start_tick);
                        state.dragged_clip = Some((
                            clip.id,
                            zone,
                            clip.track_index,
                            clip.start_tick,
                            clip.length_ticks,
                            offset_ticks,
                        ));
                        state.is_scrubbing_playhead = false;

                        return Some(Action::publish(UiMessage::SelectRegion(clip.id)));
                    } else {
                        // Clicked empty lane: start marquee drag and snap playhead
                        let snapped_tick = self.snap.snap(target_tick, self.horizontal_zoom);
                        state.marquee_drag = Some((cursor_pos, cursor_pos));
                        return Some(Action::publish(UiMessage::SetPlayhead(snapped_tick)));
                    }
                }

                None
            }

            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_scrubbing_playhead = false;
                state.cycle_drag = None;

                // Handle Marquee Selection Completion
                if let Some((start_pt, end_pt)) = state.marquee_drag.take() {
                    let dx = (end_pt.x - start_pt.x).abs();
                    let dy = (end_pt.y - start_pt.y).abs();
                    if dx > 8.0 || dy > 8.0 {
                        let t1 = self.x_to_tick(start_pt.x.min(end_pt.x), bounds.width);
                        let t2 = self.x_to_tick(start_pt.x.max(end_pt.x), bounds.width);
                        let trk1 = self.y_to_track(start_pt.y.min(end_pt.y)).unwrap_or(0);
                        let trk2 = self.y_to_track(start_pt.y.max(end_pt.y)).unwrap_or(trk1);
                        return Some(Action::publish(UiMessage::TimelineMarqueeSelect {
                            start_tick: t1,
                            end_tick: t2,
                            start_track: trk1.min(trk2),
                            end_track: trk1.max(trk2),
                        }));
                    }
                }

                if let Some((clip_id, zone, orig_trk, orig_start, orig_len, offset_ticks)) =
                    state.dragged_clip.take()
                {
                    let target_tick = self.x_to_tick(cursor_pos.x, bounds.width);
                    match zone {
                        SmartToolZone::GrabMove => {
                            let target_trk = self.y_to_track(cursor_pos.y).unwrap_or(orig_trk);
                            let new_start = target_tick.saturating_sub(offset_ticks);
                            let snapped_tick = self.snap.snap(new_start, self.horizontal_zoom);
                            return Some(Action::publish(UiMessage::TimelineMoveClip {
                                clip_id,
                                new_track_id: target_trk,
                                new_start_tick: snapped_tick,
                            }));
                        }
                        SmartToolZone::TrimStart => {
                            let snapped_tick = self.snap.snap(target_tick, self.horizontal_zoom);
                            if snapped_tick < orig_start + orig_len.saturating_sub(120) {
                                return Some(Action::publish(UiMessage::TimelineTrimClipStart {
                                    clip_id,
                                    new_start_tick: snapped_tick,
                                }));
                            }
                        }
                        SmartToolZone::TrimEnd => {
                            let snapped_tick = self.snap.snap(target_tick, self.horizontal_zoom);
                            if snapped_tick > orig_start + 120 {
                                let new_len = snapped_tick.saturating_sub(orig_start);
                                return Some(Action::publish(UiMessage::TimelineTrimClipEnd {
                                    clip_id,
                                    new_length_ticks: new_len,
                                }));
                            }
                        }
                        SmartToolZone::FadeIn => {
                            let fade_ticks = target_tick.saturating_sub(orig_start);
                            return Some(Action::publish(UiMessage::TimelineFadeClip {
                                clip_id,
                                fade_in_ticks: Some(fade_ticks),
                                fade_out_ticks: None,
                            }));
                        }
                        SmartToolZone::FadeOut => {
                            let fade_ticks = (orig_start + orig_len).saturating_sub(target_tick);
                            return Some(Action::publish(UiMessage::TimelineFadeClip {
                                clip_id,
                                fade_in_ticks: None,
                                fade_out_ticks: Some(fade_ticks),
                            }));
                        }
                        SmartToolZone::SelectRange => {}
                    }
                }
                None
            }

            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let target_tick = self.x_to_tick(cursor_pos.x, bounds.width);

                if state.is_scrubbing_playhead {
                    return Some(Action::publish(UiMessage::SetPlayhead(target_tick)));
                }

                if let Some((_, ref mut curr)) = state.marquee_drag {
                    *curr = cursor_pos;
                }

                if let Some((mode, start_anchor, end_anchor)) = state.cycle_drag {
                    let snapped = self.snap.snap(target_tick, self.horizontal_zoom);
                    match mode {
                        CycleDragMode::AdjustStart => {
                            if snapped < end_anchor.saturating_sub(120) {
                                return Some(Action::publish(UiMessage::SetCycleRange {
                                    start_tick: snapped,
                                    end_tick: end_anchor,
                                }));
                            }
                        }
                        CycleDragMode::AdjustEnd => {
                            if snapped > start_anchor + 120 {
                                return Some(Action::publish(UiMessage::SetCycleRange {
                                    start_tick: start_anchor,
                                    end_tick: snapped,
                                }));
                            }
                        }
                        CycleDragMode::MoveEntire => {
                            let len = end_anchor.saturating_sub(start_anchor);
                            return Some(Action::publish(UiMessage::SetCycleRange {
                                start_tick: snapped,
                                end_tick: snapped + len,
                            }));
                        }
                    }
                }
                None
            }

            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll_y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 20.0,
                };
                let new_zoom = (self.horizontal_zoom + scroll_y * 0.15).clamp(0.2, 10.0);
                Some(Action::publish(UiMessage::SetHorizontalZoom(new_zoom)))
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

        // 1. Overall Dark Background
        let base_bg = if self.is_fl_theme {
            Color::from_rgb8(0x15, 0x15, 0x18)
        } else {
            Color::from_rgb8(0x13, 0x14, 0x18)
        };
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            base_bg,
        );

        // 2. Track Lanes (Alternating Charcoal & Slate)
        for i in 0..self.track_count {
            let y = self.track_to_y(i);
            if y + LANE_HEIGHT < RULER_HEIGHT || y > bounds.height {
                continue;
            }

            let bg_color = if i == self.selected_track_id {
                if self.is_fl_theme {
                    Color::from_rgb8(0x2a, 0x22, 0x1c)
                } else {
                    Color::from_rgb8(0x22, 0x27, 0x33) // Highlighted selected track
                }
            } else if i % 2 == 0 {
                Color::from_rgb8(0x1b, 0x1d, 0x22)
            } else {
                Color::from_rgb8(0x20, 0x23, 0x29)
            };

            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(bounds.width, LANE_HEIGHT - 1.0),
                bg_color,
            );

            // 1px Horizontal Lane Separator Divider
            let divider = Path::line(
                Point::new(0.0, y + LANE_HEIGHT - 1.0),
                Point::new(bounds.width, y + LANE_HEIGHT - 1.0),
            );
            frame.stroke(
                &divider,
                Stroke::default()
                    .with_color(Color::from_rgb8(0x13, 0x14, 0x18))
                    .with_width(1.0),
            );
        }

        // 3. Vertical Grid Lines (Bars, Beats, Sub-divisions)
        let mut tick = self.visible_start_tick;
        while tick <= self.visible_end_tick {
            let x = self.tick_to_x(tick, bounds.width);
            if x <= bounds.width {
                if tick.is_multiple_of(TICKS_PER_BAR) {
                    // Major Bar Line across lanes
                    let line =
                        Path::line(Point::new(x, RULER_HEIGHT), Point::new(x, bounds.height));
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(Color::from_rgba8(0x40, 0x46, 0x54, 0.6))
                            .with_width(1.0),
                    );
                } else if tick.is_multiple_of(PPQN) {
                    // Quarter Note Beat Line
                    let line =
                        Path::line(Point::new(x, RULER_HEIGHT), Point::new(x, bounds.height));
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(Color::from_rgba8(0x28, 0x2c, 0x36, 0.4))
                            .with_width(1.0),
                    );
                } else if self.horizontal_zoom >= 1.2 && tick.is_multiple_of(240) {
                    // 16th Sub-division Line
                    let line =
                        Path::line(Point::new(x, RULER_HEIGHT), Point::new(x, bounds.height));
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(Color::from_rgba8(0x20, 0x23, 0x2b, 0.25))
                            .with_width(0.8),
                    );
                }
            }
            tick += 240;
        }

        // 4. Logic Pro Region Cards (Audio, MIDI, Pattern)
        for clip in &self.clips {
            if clip.track_index < self.track_count {
                let y = self.track_to_y(clip.track_index) + 4.0;
                let h = LANE_HEIGHT - 8.0;
                let x_start = self.tick_to_x(clip.start_tick, bounds.width);
                let x_end = self.tick_to_x(clip.start_tick + clip.length_ticks, bounds.width);
                let w = (x_end - x_start).max(6.0);

                if x_start < bounds.width && x_end > 0.0 && y + h > RULER_HEIGHT && y < bounds.height {
                    let (base_color, glyph) = match clip.kind {
                        ClipVisualKind::Audio { .. } => (
                            Color::from_rgb8(
                                clip.color_rgb[0].clamp(20, 180),
                                clip.color_rgb[1].clamp(40, 200),
                                clip.color_rgb[2].clamp(80, 240),
                            ),
                            "≈",
                        ),
                        ClipVisualKind::Midi { .. } => (
                            Color::from_rgb8(0x1e, 0x6e, 0x38), // Logic Emerald Green
                            "♩",
                        ),
                        ClipVisualKind::Pattern => (
                            Color::from_rgb8(0x5c, 0x2b, 0x8c), // Logic Electric Purple
                            "⊞",
                        ),
                    };

                    let effective_color = if clip.muted {
                        Color::from_rgba(
                            base_color.r * 0.5,
                            base_color.g * 0.5,
                            base_color.b * 0.5,
                            0.5,
                        )
                    } else {
                        base_color
                    };

                    // 4a. Region Base Body (3px rounded rectangle)
                    let card_rect = Path::rectangle(Point::new(x_start, y), Size::new(w, h));
                    frame.fill(&card_rect, effective_color);

                    // 4b. Top Region Header Bar (16px translucent overlay)
                    let header_h = 16.0;
                    frame.fill_rectangle(
                        Point::new(x_start, y),
                        Size::new(w, header_h),
                        Color::from_rgba8(0, 0, 0, 0.40),
                    );

                    // Region Header Text: Glyph + Name
                    let display_title = if clip.muted {
                        format!("{} {} [MUTED]", glyph, clip.name)
                    } else {
                        format!("{} {}", glyph, clip.name)
                    };
                    frame.fill_text(canvas::Text {
                        content: display_title,
                        position: Point::new(x_start + 4.0, y + 2.0),
                        color: Color::from_rgb8(0xf0, 0xf3, 0xf8),
                        size: 10.0.into(),
                        ..Default::default()
                    });

                    // 4c. Interior Audio Waveform or MIDI Note Rendering
                    let body_y = y + header_h;
                    let body_h = h - header_h;
                    let mid_y = body_y + (body_h * 0.5);

                    match &clip.kind {
                        ClipVisualKind::Audio { waveform_peaks } => {
                            // Draw Centerline
                            let center_line = Path::line(
                                Point::new(x_start, mid_y),
                                Point::new(x_start + w, mid_y),
                            );
                            frame.stroke(
                                &center_line,
                                Stroke::default()
                                    .with_color(Color::from_rgba8(255, 255, 255, 0.25))
                                    .with_width(0.8),
                            );

                            // Draw Waveform Peak Contours
                            let num_bars = ((w / 3.0) as usize).clamp(4, 256);
                            for step in 0..num_bars {
                                let bar_x = x_start + (step as f32 * 3.0);
                                let peak_idx = (step * waveform_peaks.len().max(1)) / num_bars;
                                let amp = if !waveform_peaks.is_empty() {
                                    let (min, max) =
                                        waveform_peaks[peak_idx % waveform_peaks.len()];
                                    ((max - min) * 0.5).abs().clamp(0.05, 0.95)
                                } else {
                                    // Generate authentic pseudo-harmonic waveform envelope
                                    let phase = (step as f32 * 0.25).sin().abs();
                                    (phase * 0.75 + 0.15).clamp(0.05, 0.9)
                                };

                                let bar_half_h = (body_h * 0.45) * amp;
                                let peak_line = Path::line(
                                    Point::new(bar_x, mid_y - bar_half_h),
                                    Point::new(bar_x, mid_y + bar_half_h),
                                );
                                frame.stroke(
                                    &peak_line,
                                    Stroke::default()
                                        .with_color(Color::from_rgba8(230, 245, 255, 0.75))
                                        .with_width(1.5),
                                );
                            }
                        }
                        ClipVisualKind::Midi { notes } => {
                            // Draw miniature horizontal piano roll note bars
                            if notes.is_empty() {
                                // Default melodic note pattern
                                for n in 0..8 {
                                    let note_x = x_start + (n as f32 * (w / 8.0)) + 2.0;
                                    let note_w = ((w / 8.0) - 4.0).max(3.0);
                                    let note_pitch_y = mid_y + ((n as f32 % 5.0) - 2.0) * 4.0;
                                    frame.fill_rectangle(
                                        Point::new(note_x, note_pitch_y - 1.5),
                                        Size::new(note_w, 3.0),
                                        Color::from_rgba8(220, 255, 230, 0.90),
                                    );
                                }
                            } else {
                                for &(rel_tick, pitch, len_tick) in notes {
                                    let note_x =
                                        self.tick_to_x(clip.start_tick + rel_tick, bounds.width);
                                    let note_w = (self.tick_to_x(
                                        clip.start_tick + rel_tick + len_tick,
                                        bounds.width,
                                    ) - note_x)
                                        .max(3.0);
                                    let pitch_offset = ((pitch as f32 - 60.0) * 0.5)
                                        .clamp(-body_h * 0.4, body_h * 0.4);
                                    let note_y = mid_y - pitch_offset;
                                    frame.fill_rectangle(
                                        Point::new(note_x, note_y - 1.5),
                                        Size::new(note_w, 3.0),
                                        Color::from_rgba8(220, 255, 230, 0.90),
                                    );
                                }
                            }
                        }
                        ClipVisualKind::Pattern => {
                            // FL Studio style 16-step sequencer pattern preview blocks
                            let step_w = (w / 16.0).max(3.0);
                            for s in 0..16 {
                                let block_x = x_start + (s as f32 * step_w) + 1.0;
                                let is_even_beat = (s / 4) % 2 == 0;
                                let step_color = if is_even_beat {
                                    Color::from_rgba8(240, 240, 245, 0.85)
                                } else {
                                    Color::from_rgba8(180, 185, 200, 0.70)
                                };
                                frame.fill_rectangle(
                                    Point::new(block_x, mid_y - 4.0),
                                    Size::new(step_w - 2.0, 8.0),
                                    step_color,
                                );
                            }
                        }
                    }

                    // 4d. Fade-In and Fade-Out Curve Visuals
                    if clip.fade_in_ticks > 0 {
                        let fi_w = (self
                            .tick_to_x(clip.start_tick + clip.fade_in_ticks, bounds.width)
                            - x_start)
                            .min(w * 0.5);
                        let fade_tri = Path::new(|p| {
                            p.move_to(Point::new(x_start, body_y));
                            p.line_to(Point::new(x_start + fi_w, body_y));
                            p.line_to(Point::new(x_start, body_y + body_h));
                            p.close();
                        });
                        frame.fill(&fade_tri, Color::from_rgba8(0, 0, 0, 0.35));
                    }
                    if clip.fade_out_ticks > 0 {
                        let fo_w = (x_end
                            - self.tick_to_x(
                                (clip.start_tick + clip.length_ticks)
                                    .saturating_sub(clip.fade_out_ticks),
                                bounds.width,
                            ))
                        .min(w * 0.5);
                        let fade_tri = Path::new(|p| {
                            p.move_to(Point::new(x_end, body_y));
                            p.line_to(Point::new(x_end - fo_w, body_y));
                            p.line_to(Point::new(x_end, body_y + body_h));
                            p.close();
                        });
                        frame.fill(&fade_tri, Color::from_rgba8(0, 0, 0, 0.35));
                    }

                    // 4e. Loop Notch Indicator (Logic Pro rounded corner loop notch)
                    if clip.looped {
                        let notch = Path::circle(Point::new(x_end - 4.0, y + 4.0), 3.0);
                        frame.fill(&notch, Color::from_rgb8(0xff, 0xff, 0xff));
                    }

                    // 4f. Region Outline Border & Selection Ring
                    let outline = Path::rectangle(Point::new(x_start, y), Size::new(w, h));
                    let (border_color, border_w) = if clip.is_selected {
                        if self.is_fl_theme {
                            (Color::from_rgb8(0xf8, 0x8a, 0x36), 2.0) // FL Studio Orange Glowing Selection
                        } else {
                            (Color::from_rgb8(0x00, 0x7a, 0xff), 2.0) // Apple Blue Glowing Selection
                        }
                    } else {
                        (Color::from_rgba8(255, 255, 255, 0.35), 1.0)
                    };
                    frame.stroke(
                        &outline,
                        Stroke::default()
                            .with_color(border_color)
                            .with_width(border_w),
                    );
                }
            }
        }

        // 4g. Active Marquee Drag or Selection Box
        if let Some((start_pt, end_pt)) = state.marquee_drag {
            let rx = start_pt.x.min(end_pt.x);
            let ry = start_pt.y.min(end_pt.y);
            let rw = (end_pt.x - start_pt.x).abs();
            let rh = (end_pt.y - start_pt.y).abs();
            let marquee_color = if self.is_fl_theme {
                Color::from_rgba8(0xf8, 0x8a, 0x36, 0.20)
            } else {
                Color::from_rgba8(0x00, 0x7a, 0xff, 0.20)
            };
            let marquee_border = if self.is_fl_theme {
                Color::from_rgb8(0xf8, 0x8a, 0x36)
            } else {
                Color::from_rgb8(0x00, 0x7a, 0xff)
            };
            frame.fill_rectangle(Point::new(rx, ry), Size::new(rw, rh), marquee_color);
            let border_rect = Path::rectangle(Point::new(rx, ry), Size::new(rw, rh));
            frame.stroke(
                &border_rect,
                Stroke::default().with_color(marquee_border).with_width(1.2),
            );
        } else if let Some((t1, t2, trk1, trk2)) = self.marquee_selection {
            let x1 = self.tick_to_x(t1, bounds.width);
            let x2 = self.tick_to_x(t2, bounds.width);
            let y1 = self.track_to_y(trk1);
            let y2 = self.track_to_y(trk2) + LANE_HEIGHT;
            let rx = x1.min(x2);
            let ry = y1.min(y2);
            let rw = (x2 - x1).abs();
            let rh = (y2 - y1).abs();
            let marquee_color = if self.is_fl_theme {
                Color::from_rgba8(0xf8, 0x8a, 0x36, 0.15)
            } else {
                Color::from_rgba8(0x00, 0x7a, 0xff, 0.15)
            };
            let marquee_border = if self.is_fl_theme {
                Color::from_rgb8(0xf8, 0x8a, 0x36)
            } else {
                Color::from_rgb8(0x00, 0x7a, 0xff)
            };
            frame.fill_rectangle(Point::new(rx, ry), Size::new(rw, rh), marquee_color);
            let border_rect = Path::rectangle(Point::new(rx, ry), Size::new(rw, rh));
            frame.stroke(
                &border_rect,
                Stroke::default().with_color(marquee_border).with_width(1.0),
            );
        }

        // 5. Logic Pro Top Ruler Bar (Height = 30.0)
        // Ruler Background
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(bounds.width, RULER_HEIGHT),
            Color::from_rgb8(0x1a, 0x1c, 0x22),
        );
        // Ruler Bottom 1px Border
        let ruler_border = Path::line(
            Point::new(0.0, RULER_HEIGHT - 0.5),
            Point::new(bounds.width, RULER_HEIGHT - 0.5),
        );
        frame.stroke(
            &ruler_border,
            Stroke::default()
                .with_color(Color::from_rgb8(0x0e, 0x10, 0x14))
                .with_width(1.0),
        );

        // 5a. Logic Pro Signature Amber Cycle Bar (Top 12px) / FL Orange Cycle Bar
        if let Some((start_tick, end_tick)) = self.loop_range {
            let start_x = self.tick_to_x(start_tick, bounds.width);
            let end_x = self.tick_to_x(end_tick, bounds.width);
            if end_x > start_x {
                let cycle_color = if self.is_fl_theme {
                    Color::from_rgb8(0xf8, 0x8a, 0x36)
                } else {
                    Color::from_rgb8(0xe5, 0xa9, 0x3c)
                };
                frame.fill_rectangle(
                    Point::new(start_x, 1.0),
                    Size::new(end_x - start_x, 11.0),
                    cycle_color,
                );

                // Start Bracket Handle '['
                let left_bracket = Path::line(
                    Point::new(start_x + 1.0, 1.0),
                    Point::new(start_x + 1.0, 12.0),
                );
                frame.stroke(
                    &left_bracket,
                    Stroke::default().with_color(Color::WHITE).with_width(2.0),
                );

                // Right Bracket Handle ']'
                let right_bracket =
                    Path::line(Point::new(end_x - 1.0, 1.0), Point::new(end_x - 1.0, 12.0));
                frame.stroke(
                    &right_bracket,
                    Stroke::default().with_color(Color::WHITE).with_width(2.0),
                );

                // Cycle Bar Label
                let bar_start = (start_tick / TICKS_PER_BAR) + 1;
                let bar_end = (end_tick / TICKS_PER_BAR) + 1;
                frame.fill_text(canvas::Text {
                    content: format!("CYCLE [{}-{}]", bar_start, bar_end),
                    position: Point::new(start_x + 6.0, 2.0),
                    color: Color::from_rgb8(0x20, 0x15, 0x05),
                    size: 8.0.into(),
                    ..Default::default()
                });
            }
        }

        // 5b. Ruler Bar Numbers and Beat Hash Marks
        let mut ruler_tick = self.visible_start_tick;
        while ruler_tick <= self.visible_end_tick {
            let x = self.tick_to_x(ruler_tick, bounds.width);
            if x <= bounds.width {
                if ruler_tick.is_multiple_of(TICKS_PER_BAR) {
                    let bar_num = (ruler_tick / TICKS_PER_BAR) + 1;
                    // Major Tick Line in Ruler
                    let tick_line = Path::line(Point::new(x, 15.0), Point::new(x, RULER_HEIGHT));
                    frame.stroke(
                        &tick_line,
                        Stroke::default()
                            .with_color(Color::from_rgb8(0xd1, 0xd5, 0xdb))
                            .with_width(1.2),
                    );

                    // Bar Number Text
                    frame.fill_text(canvas::Text {
                        content: format!("{}", bar_num),
                        position: Point::new(x + 3.0, 14.0),
                        color: Color::from_rgb8(0xd1, 0xd5, 0xdb),
                        size: 10.0.into(),
                        ..Default::default()
                    });
                } else if ruler_tick.is_multiple_of(PPQN) {
                    // Beat Hash Mark
                    let tick_line = Path::line(Point::new(x, 22.0), Point::new(x, RULER_HEIGHT));
                    frame.stroke(
                        &tick_line,
                        Stroke::default()
                            .with_color(Color::from_rgb8(0x6b, 0x72, 0x80))
                            .with_width(0.8),
                    );
                }
            }
            ruler_tick += PPQN;
        }

        // 6. Logic Pro Iconic Playhead
        let playhead_x = self.tick_to_x(self.playhead_tick, bounds.width);

        // 6a. Playhead Downward Triangle Head in Ruler
        let triangle_head = Path::new(|p| {
            p.move_to(Point::new(playhead_x - 6.0, 2.0));
            p.line_to(Point::new(playhead_x + 6.0, 2.0));
            p.line_to(Point::new(playhead_x, RULER_HEIGHT - 2.0));
            p.close();
        });
        frame.fill(&triangle_head, Color::WHITE);

        // 6b. Playhead Laser Guide Line extending across all lanes
        let playhead_laser = Path::line(
            Point::new(playhead_x, RULER_HEIGHT),
            Point::new(playhead_x, bounds.height),
        );
        frame.stroke(
            &playhead_laser,
            Stroke::default().with_color(Color::WHITE).with_width(1.5),
        );

        vec![frame.into_geometry()]
    }
}

pub fn view_canvas_timeline<'a>(program: ArrangementCanvasProgram) -> Element<'a, UiMessage> {
    Canvas::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
