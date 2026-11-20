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
    AutomationTension(usize),
    AutomationNode(usize),
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
    Automation {
        parameter_target: String,
        points: Vec<(u64, f32, f32)>, // (tick_offset, normalized_val 0..1, tension -1..1)
    },
}

/// Evaluates FL Studio Hermite spline tension curvature for normalized progression u in [0, 1]
pub fn evaluate_spline_tension(u: f32, tension: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    let t = tension.clamp(-1.0, 1.0);
    if t > 0.0 {
        u.powf(1.0 + t * 3.0)
    } else if t < 0.0 {
        1.0 - (1.0 - u).powf(1.0 - t * 3.0)
    } else {
        u
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct LiveDragPreview {
    pub clip_id: usize,
    pub zone: SmartToolZone,
    pub target_track: usize,
    pub target_start_tick: u64,
    pub target_length_ticks: u64,
    pub delta_ticks: i64,
    pub is_valid_drop: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CanvasInteractionState {
    pub is_scrubbing_playhead: bool,
    pub cycle_drag: Option<(CycleDragMode, u64, u64)>, // mode, start_anchor, end_anchor
    pub dragged_clip: Option<(usize, SmartToolZone, usize, u64, u64, u64)>, // clip_id, zone, orig_track, orig_start, orig_len, offset_ticks
    pub live_drag: Option<LiveDragPreview>,
    pub marquee_drag: Option<(Point, Point)>, // start_anchor, current_anchor
    pub last_cursor_pos: Option<Point>,
}

pub struct ArrangementCanvasProgram {
    pub playhead_tick: u64,
    pub visible_start_tick: u64,
    pub visible_end_tick: u64,
    pub horizontal_zoom: f32,
    pub track_count: usize,
    pub track_types: Vec<crate::timeline::track::TrackType>,
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
            track_types: Vec::new(),
            loop_range: Some((0, TICKS_PER_BAR * 4)),
            clips: Vec::new(),
            selected_track_id: 0,
            snap: SnapGrid::Sixteenth,
            scroll_offset_y: 0.0,
            marquee_selection: None,
            is_fl_theme: false,
        }
    }

    pub fn with_track_types(mut self, track_types: Vec<crate::timeline::track::TrackType>) -> Self {
        self.track_types = track_types;
        self
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
                        let mut auto_zone = None;
                        if let ClipVisualKind::Automation { points, .. } = &clip.kind {
                            // Check if cursor is near an automation node
                            for (idx, p) in points.iter().enumerate() {
                                let px = self.tick_to_x(clip.start_tick + p.0, bounds.width);
                                let py = (y_clip + h_clip) - p.1 * (h_clip - 4.0) - 2.0;
                                let dist_sq =
                                    (cursor_pos.x - px).powi(2) + (cursor_pos.y - py).powi(2);
                                if dist_sq <= 64.0 {
                                    auto_zone = Some(SmartToolZone::AutomationNode(idx));
                                    break;
                                }
                            }

                            // If not a node, check if cursor is near a midpoint tension handle
                            if auto_zone.is_none() {
                                for i in 0..points.len().saturating_sub(1) {
                                    let p0 = &points[i];
                                    let p1 = &points[i + 1];
                                    let t0 = clip.start_tick + p0.0;
                                    let t1 = clip.start_tick + p1.0;
                                    let x0 = self.tick_to_x(t0, bounds.width);
                                    let x1 = self.tick_to_x(t1, bounds.width);
                                    let y0 = (y_clip + h_clip) - p0.1 * (h_clip - 4.0) - 2.0;
                                    let y1 = (y_clip + h_clip) - p1.1 * (h_clip - 4.0) - 2.0;
                                    let mid_prog = evaluate_spline_tension(0.5, p0.2);
                                    let mx = x0 + (x1 - x0) * 0.5;
                                    let my = y0 + (y1 - y0) * mid_prog;
                                    let dist_sq =
                                        (cursor_pos.x - mx).powi(2) + (cursor_pos.y - my).powi(2);
                                    if dist_sq <= 64.0 {
                                        auto_zone = Some(SmartToolZone::AutomationTension(i));
                                        break;
                                    }
                                }
                            }
                        }

                        let zone = if let Some(az) = auto_zone {
                            az
                        } else if cursor_pos.x - x_start < 8.0 {
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
                        state.live_drag = Some(LiveDragPreview {
                            clip_id: clip.id,
                            zone,
                            target_track: clip.track_index,
                            target_start_tick: clip.start_tick,
                            target_length_ticks: clip.length_ticks,
                            delta_ticks: 0,
                            is_valid_drop: true,
                        });
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
                let preview = state.live_drag.take();

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
                            let is_valid =
                                preview.as_ref().map(|p| p.is_valid_drop).unwrap_or(true);
                            let target_trk = if is_valid {
                                self.y_to_track(cursor_pos.y).unwrap_or(orig_trk)
                            } else {
                                orig_trk
                            };
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
                        SmartToolZone::AutomationTension(seg_idx) => {
                            let y_clip = RULER_HEIGHT + (orig_trk as f32 * LANE_HEIGHT) + 4.0;
                            let h_clip = LANE_HEIGHT - 8.0;
                            let mid_y = y_clip + h_clip * 0.5;
                            let tension =
                                ((mid_y - cursor_pos.y) / (h_clip * 0.35)).clamp(-1.0, 1.0);
                            return Some(Action::publish(
                                UiMessage::TimelineSetAutomationTension {
                                    clip_id,
                                    segment_index: seg_idx,
                                    tension,
                                },
                            ));
                        }
                        SmartToolZone::AutomationNode(node_idx) => {
                            let y_clip = RULER_HEIGHT + (orig_trk as f32 * LANE_HEIGHT) + 4.0;
                            let h_clip = LANE_HEIGHT - 8.0;
                            let value =
                                ((y_clip + h_clip - cursor_pos.y) / (h_clip - 4.0)).clamp(0.0, 1.0);
                            return Some(Action::publish(UiMessage::TimelineSetAutomationNode {
                                clip_id,
                                node_index: node_idx,
                                value,
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

                // Live Clip Dragging & Snapping Guide Preview
                if let Some((clip_id, zone, orig_trk, orig_start, orig_len, offset_ticks)) =
                    state.dragged_clip
                {
                    let target_trk = self.y_to_track(cursor_pos.y).unwrap_or(orig_trk);
                    let is_audio = self
                        .clips
                        .iter()
                        .find(|c| c.id == clip_id)
                        .map(|c| matches!(c.kind, ClipVisualKind::Audio { .. }))
                        .unwrap_or(true);

                    let is_compatible = if self.is_fl_theme {
                        true // FL Studio Free-Form Playlist Track Coexistence
                    } else if let Some(&dest_type) = self.track_types.get(target_trk) {
                        if is_audio {
                            matches!(
                                dest_type,
                                crate::timeline::track::TrackType::Audio
                                    | crate::timeline::track::TrackType::Aux
                            )
                        } else {
                            matches!(
                                dest_type,
                                crate::timeline::track::TrackType::Software
                                    | crate::timeline::track::TrackType::Midi
                                    | crate::timeline::track::TrackType::Drum
                            )
                        }
                    } else {
                        true
                    };

                    match zone {
                        SmartToolZone::GrabMove => {
                            let new_start = target_tick.saturating_sub(offset_ticks);
                            let snapped_tick = self.snap.snap(new_start, self.horizontal_zoom);
                            let delta = snapped_tick as i64 - orig_start as i64;
                            state.live_drag = Some(LiveDragPreview {
                                clip_id,
                                zone,
                                target_track: target_trk,
                                target_start_tick: snapped_tick,
                                target_length_ticks: orig_len,
                                delta_ticks: delta,
                                is_valid_drop: is_compatible,
                            });
                        }
                        SmartToolZone::TrimStart => {
                            let snapped_tick = self.snap.snap(target_tick, self.horizontal_zoom);
                            if snapped_tick < orig_start + orig_len.saturating_sub(120) {
                                let new_len = (orig_start + orig_len).saturating_sub(snapped_tick);
                                let delta = snapped_tick as i64 - orig_start as i64;
                                state.live_drag = Some(LiveDragPreview {
                                    clip_id,
                                    zone,
                                    target_track: orig_trk,
                                    target_start_tick: snapped_tick,
                                    target_length_ticks: new_len,
                                    delta_ticks: delta,
                                    is_valid_drop: true,
                                });
                            }
                        }
                        SmartToolZone::TrimEnd => {
                            let snapped_tick = self.snap.snap(target_tick, self.horizontal_zoom);
                            if snapped_tick > orig_start + 120 {
                                let new_len = snapped_tick.saturating_sub(orig_start);
                                let delta = new_len as i64 - orig_len as i64;
                                state.live_drag = Some(LiveDragPreview {
                                    clip_id,
                                    zone,
                                    target_track: orig_trk,
                                    target_start_tick: orig_start,
                                    target_length_ticks: new_len,
                                    delta_ticks: delta,
                                    is_valid_drop: true,
                                });
                            }
                        }
                        SmartToolZone::AutomationTension(_) | SmartToolZone::AutomationNode(_) => {
                            state.live_drag = Some(LiveDragPreview {
                                clip_id,
                                zone,
                                target_track: orig_trk,
                                target_start_tick: orig_start,
                                target_length_ticks: orig_len,
                                delta_ticks: 0,
                                is_valid_drop: true,
                            });
                        }
                        _ => {}
                    }
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
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), base_bg);

        // 2. Track Lanes (Alternating Charcoal & Slate with Hardware Rack Depth)
        for i in 0..self.track_count {
            let y = self.track_to_y(i);
            if y + LANE_HEIGHT < RULER_HEIGHT || y > bounds.height {
                continue;
            }

            let bg_color = if i == self.selected_track_id {
                if self.is_fl_theme {
                    Color::from_rgb8(0x2d, 0x24, 0x1d)
                } else {
                    Color::from_rgb8(0x23, 0x2a, 0x36) // Highlighted selected track
                }
            } else if self.is_fl_theme {
                // Calibrated to PLGridback: 5129268 / #34444e base shading
                if i % 2 == 0 {
                    Color::from_rgb8(0x21, 0x25, 0x2d)
                } else {
                    Color::from_rgb8(0x26, 0x2b, 0x34)
                }
            } else if i % 2 == 0 {
                Color::from_rgb8(0x1a, 0x1c, 0x22)
            } else {
                Color::from_rgb8(0x1f, 0x22, 0x28)
            };

            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(bounds.width, LANE_HEIGHT - 1.0),
                bg_color,
            );

            // 1px Top Highlight for Hardware Rack Bevel Depth
            let top_highlight = Path::line(Point::new(0.0, y), Point::new(bounds.width, y));
            frame.stroke(
                &top_highlight,
                Stroke::default()
                    .with_color(Color::from_rgba8(255, 255, 255, 0.03))
                    .with_width(1.0),
            );

            // Left Indicator Strip for Selected Track
            if i == self.selected_track_id {
                let accent_col = if self.is_fl_theme {
                    Color::from_rgb8(0xf8, 0x8a, 0x36)
                } else {
                    Color::from_rgb8(0x00, 0x7a, 0xff)
                };
                frame.fill_rectangle(
                    Point::new(0.0, y),
                    Size::new(3.0, LANE_HEIGHT - 1.0),
                    accent_col,
                );
            }

            // 1px Horizontal Bottom Groove Divider
            let divider = Path::line(
                Point::new(0.0, y + LANE_HEIGHT - 1.0),
                Point::new(bounds.width, y + LANE_HEIGHT - 1.0),
            );
            frame.stroke(
                &divider,
                Stroke::default()
                    .with_color(Color::from_rgba8(0, 0, 0, 0.45))
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

                if x_start < bounds.width
                    && x_end > 0.0
                    && y + h > RULER_HEIGHT
                    && y < bounds.height
                {
                    let (base_color, glyph) = match &clip.kind {
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
                        ClipVisualKind::Automation { .. } => (
                            Color::from_rgb8(0xd3, 0x54, 0x00), // FL Studio Warm Copper / Amber
                            "📈",
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

                    // 4a. Region Base Body (4px rounded rectangle)
                    let card_rect = Path::rectangle(Point::new(x_start, y), Size::new(w, h));
                    frame.fill(&card_rect, effective_color);

                    // 1px Glossy Top Acrylic Highlight
                    let gloss_line = Path::line(Point::new(x_start, y), Point::new(x_start + w, y));
                    frame.stroke(
                        &gloss_line,
                        Stroke::default()
                            .with_color(Color::from_rgba8(255, 255, 255, 0.25))
                            .with_width(1.0),
                    );

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

                            // Draw Waveform with Filled RMS Energy Silhouette + Peak Contours
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

                                // Translucent filled RMS energy block
                                frame.fill_rectangle(
                                    Point::new(bar_x - 0.5, mid_y - bar_half_h),
                                    Size::new(2.0, bar_half_h * 2.0),
                                    Color::from_rgba8(255, 255, 255, 0.18),
                                );

                                // Crisp outer peak line
                                let peak_line = Path::line(
                                    Point::new(bar_x, mid_y - bar_half_h),
                                    Point::new(bar_x, mid_y + bar_half_h),
                                );
                                frame.stroke(
                                    &peak_line,
                                    Stroke::default()
                                        .with_color(Color::from_rgba8(240, 250, 255, 0.85))
                                        .with_width(1.2),
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
                        ClipVisualKind::Automation {
                            parameter_target: _,
                            points,
                        } => {
                            if points.len() >= 2 {
                                let curve_steps = 20;
                                let mut poly_points =
                                    Vec::with_capacity(points.len() * curve_steps + 2);
                                let base_y = body_y + body_h;

                                let p_first = &points[0];
                                let x_first = (self
                                    .tick_to_x(clip.start_tick + p_first.0, bounds.width))
                                .clamp(x_start, x_end);
                                poly_points.push(Point::new(x_first, base_y));

                                for i in 0..points.len().saturating_sub(1) {
                                    let p0 = &points[i];
                                    let p1 = &points[i + 1];
                                    let t0 = clip.start_tick + p0.0;
                                    let t1 = clip.start_tick + p1.0;
                                    let x0 = self.tick_to_x(t0, bounds.width);
                                    let x1 = self.tick_to_x(t1, bounds.width);
                                    let y0 = (body_y + body_h) - p0.1 * (body_h - 4.0) - 2.0;
                                    let y1 = (body_y + body_h) - p1.1 * (body_h - 4.0) - 2.0;
                                    let tension = p0.2;

                                    for step in 0..=curve_steps {
                                        let u = step as f32 / curve_steps as f32;
                                        let progress = evaluate_spline_tension(u, tension);
                                        let curr_x = x0 + (x1 - x0) * u;
                                        let curr_y = y0 + (y1 - y0) * progress;
                                        poly_points.push(Point::new(curr_x, curr_y));
                                    }
                                }

                                let p_last = points.last().unwrap();
                                let x_last = (self
                                    .tick_to_x(clip.start_tick + p_last.0, bounds.width))
                                .clamp(x_start, x_end);
                                poly_points.push(Point::new(x_last, base_y));

                                let poly_path = Path::new(|p| {
                                    if let Some(first) = poly_points.first() {
                                        p.move_to(*first);
                                        for pt in &poly_points[1..] {
                                            p.line_to(*pt);
                                        }
                                        p.close();
                                    }
                                });
                                frame.fill(&poly_path, Color::from_rgba8(255, 140, 0, 0.22));

                                // Sharp spline curve line
                                let line_path = Path::new(|p| {
                                    if poly_points.len() > 2 {
                                        p.move_to(poly_points[1]);
                                        for pt in &poly_points[2..poly_points.len() - 1] {
                                            p.line_to(*pt);
                                        }
                                    }
                                });
                                frame.stroke(
                                    &line_path,
                                    Stroke::default()
                                        .with_color(Color::from_rgb8(255, 190, 70))
                                        .with_width(2.0),
                                );

                                // Node circles and midpoint tension handles
                                for i in 0..points.len().saturating_sub(1) {
                                    let p0 = &points[i];
                                    let p1 = &points[i + 1];
                                    let t0 = clip.start_tick + p0.0;
                                    let t1 = clip.start_tick + p1.0;
                                    let x0 = self.tick_to_x(t0, bounds.width);
                                    let x1 = self.tick_to_x(t1, bounds.width);
                                    let y0 = (body_y + body_h) - p0.1 * (body_h - 4.0) - 2.0;
                                    let y1 = (body_y + body_h) - p1.1 * (body_h - 4.0) - 2.0;
                                    let tension = p0.2;

                                    // White node circle at p0
                                    let node_circ = Path::circle(Point::new(x0, y0), 3.5);
                                    frame.fill(&node_circ, Color::WHITE);
                                    frame.stroke(
                                        &node_circ,
                                        Stroke::default()
                                            .with_color(Color::from_rgb8(20, 20, 25))
                                            .with_width(1.0),
                                    );

                                    // Midpoint round tension handle
                                    let mid_prog = evaluate_spline_tension(0.5, tension);
                                    let mid_x = x0 + (x1 - x0) * 0.5;
                                    let mid_y_pt = y0 + (y1 - y0) * mid_prog;
                                    let handle_circ =
                                        Path::circle(Point::new(mid_x, mid_y_pt), 3.0);
                                    frame.fill(&handle_circ, Color::from_rgb8(255, 120, 0));
                                    frame.stroke(
                                        &handle_circ,
                                        Stroke::default().with_color(Color::WHITE).with_width(1.0),
                                    );
                                }
                                if let Some(last_p) = points.last() {
                                    let x_l =
                                        self.tick_to_x(clip.start_tick + last_p.0, bounds.width);
                                    let y_l = (body_y + body_h) - last_p.1 * (body_h - 4.0) - 2.0;
                                    let node_circ = Path::circle(Point::new(x_l, y_l), 3.5);
                                    frame.fill(&node_circ, Color::WHITE);
                                    frame.stroke(
                                        &node_circ,
                                        Stroke::default()
                                            .with_color(Color::from_rgb8(20, 20, 25))
                                            .with_width(1.0),
                                    );
                                }
                            }
                        }
                    }

                    // 4d. Fade-In and Fade-Out Curve Visuals with Tactile Handles
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

                        // Tactile node handle
                        let handle = Path::circle(Point::new(x_start + fi_w, body_y + 2.0), 2.5);
                        frame.fill(&handle, Color::from_rgba8(255, 255, 255, 0.85));
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

                        // Tactile node handle
                        let handle = Path::circle(Point::new(x_end - fo_w, body_y + 2.0), 2.5);
                        frame.fill(&handle, Color::from_rgba8(255, 255, 255, 0.85));
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

        // 4h. Live Drag Ghost Clip & Snap Guides
        if let Some(ref drag) = state.live_drag {
            let snap_x = self.tick_to_x(drag.target_start_tick, bounds.width);
            let ghost_x_end = self.tick_to_x(
                drag.target_start_tick + drag.target_length_ticks,
                bounds.width,
            );
            let ghost_w = (ghost_x_end - snap_x).max(6.0);
            let ghost_y = self.track_to_y(drag.target_track) + 4.0;
            let ghost_h = LANE_HEIGHT - 8.0;

            // 1. Vertical Beat/Bar Alignment Snap Guide Line across whole canvas
            let snap_guide = Path::line(
                Point::new(snap_x, RULER_HEIGHT),
                Point::new(snap_x, bounds.height),
            );
            frame.stroke(
                &snap_guide,
                Stroke::default()
                    .with_color(if drag.is_valid_drop {
                        Color::from_rgba8(0x00, 0xe5, 0xff, 0.75) // Cyan laser guide
                    } else {
                        Color::from_rgba8(0xff, 0x47, 0x57, 0.75) // Red invalid drop guide
                    })
                    .with_width(1.5),
            );

            // 2. Translucent Ghost Box on Target Track
            let (fill_col, stroke_col) = if !drag.is_valid_drop {
                (
                    Color::from_rgba8(235, 77, 75, 0.35),
                    Color::from_rgb8(255, 71, 87),
                )
            } else if self.is_fl_theme {
                (
                    Color::from_rgba8(248, 138, 54, 0.40),
                    Color::from_rgb8(255, 175, 80),
                )
            } else {
                (
                    Color::from_rgba8(0, 122, 255, 0.40),
                    Color::from_rgb8(100, 200, 255),
                )
            };

            let ghost_rect =
                Path::rectangle(Point::new(snap_x, ghost_y), Size::new(ghost_w, ghost_h));
            frame.fill(&ghost_rect, fill_col);
            frame.stroke(
                &ghost_rect,
                Stroke::default().with_color(stroke_col).with_width(2.0),
            );

            // 3. Floating HUD Information Pill
            let bar = (drag.target_start_tick / TICKS_PER_BAR) + 1;
            let beat = ((drag.target_start_tick % TICKS_PER_BAR) / PPQN) + 1;
            let delta_txt = if drag.delta_ticks >= 0 {
                format!("+{} ticks", drag.delta_ticks)
            } else {
                format!("{} ticks", drag.delta_ticks)
            };

            let hud_label = match drag.zone {
                SmartToolZone::AutomationTension(seg) => {
                    let cursor_y = state
                        .last_cursor_pos
                        .map(|p| p.y)
                        .unwrap_or(ghost_y + ghost_h * 0.5);
                    let tension =
                        ((ghost_y + ghost_h * 0.5 - cursor_y) / (ghost_h * 0.35)).clamp(-1.0, 1.0);
                    let shape = if tension > 0.05 {
                        "Convex"
                    } else if tension < -0.05 {
                        "Concave"
                    } else {
                        "Linear"
                    };
                    format!(
                        "Spline Seg {} | Tension: {:+.2} ({})",
                        seg + 1,
                        tension,
                        shape
                    )
                }
                SmartToolZone::AutomationNode(idx) => {
                    let cursor_y = state
                        .last_cursor_pos
                        .map(|p| p.y)
                        .unwrap_or(ghost_y + ghost_h * 0.5);
                    let val = ((ghost_y + ghost_h - cursor_y) / (ghost_h - 4.0)).clamp(0.0, 1.0);
                    format!(
                        "Auto Node {} | Value: {:.1}% | Bar {}.{}",
                        idx + 1,
                        val * 100.0,
                        bar,
                        beat
                    )
                }
                _ => {
                    if drag.is_valid_drop {
                        format!(
                            "Track {} | Bar {}.{} | {}",
                            drag.target_track + 1,
                            bar,
                            beat,
                            delta_txt
                        )
                    } else {
                        format!(
                            "Track {} | 🚫 Incompatible Track Type",
                            drag.target_track + 1
                        )
                    }
                }
            };

            let hud_x = snap_x.clamp(4.0, (bounds.width - 200.0).max(4.0));
            let hud_y = (ghost_y - 20.0).max(RULER_HEIGHT + 4.0);

            // HUD Background Pill
            frame.fill_rectangle(
                Point::new(hud_x, hud_y),
                Size::new(190.0, 16.0),
                Color::from_rgba8(16, 18, 24, 0.92),
            );
            let pill_outline = Path::rectangle(Point::new(hud_x, hud_y), Size::new(190.0, 16.0));
            frame.stroke(
                &pill_outline,
                Stroke::default().with_color(stroke_col).with_width(1.0),
            );

            frame.fill_text(canvas::Text {
                content: hud_label,
                position: Point::new(hud_x + 6.0, hud_y + 2.0),
                color: Color::from_rgb8(245, 248, 255),
                size: 9.0.into(),
                ..Default::default()
            });
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

                // 1px Glossy Top Shine along Cycle Bar
                let cycle_shine = Path::line(Point::new(start_x, 1.5), Point::new(end_x, 1.5));
                frame.stroke(
                    &cycle_shine,
                    Stroke::default()
                        .with_color(Color::from_rgba8(255, 255, 255, 0.45))
                        .with_width(1.0),
                );

                // Start Bracket Handle '[' with Embossed Tactile Grip
                let left_bracket = Path::line(
                    Point::new(start_x + 1.0, 1.0),
                    Point::new(start_x + 1.0, 12.0),
                );
                frame.stroke(
                    &left_bracket,
                    Stroke::default().with_color(Color::WHITE).with_width(2.0),
                );
                // Tactile rib marks
                let rib1 = Path::line(
                    Point::new(start_x + 3.0, 3.0),
                    Point::new(start_x + 3.0, 10.0),
                );
                frame.stroke(
                    &rib1,
                    Stroke::default()
                        .with_color(Color::from_rgba8(0, 0, 0, 0.35))
                        .with_width(1.0),
                );

                // Right Bracket Handle ']' with Embossed Tactile Grip
                let right_bracket =
                    Path::line(Point::new(end_x - 1.0, 1.0), Point::new(end_x - 1.0, 12.0));
                frame.stroke(
                    &right_bracket,
                    Stroke::default().with_color(Color::WHITE).with_width(2.0),
                );
                let rib2 = Path::line(Point::new(end_x - 3.0, 3.0), Point::new(end_x - 3.0, 10.0));
                frame.stroke(
                    &rib2,
                    Stroke::default()
                        .with_color(Color::from_rgba8(0, 0, 0, 0.35))
                        .with_width(1.0),
                );

                // Cycle Bar Label
                let bar_start = (start_tick / TICKS_PER_BAR) + 1;
                let bar_end = (end_tick / TICKS_PER_BAR) + 1;
                frame.fill_text(canvas::Text {
                    content: format!("CYCLE [{}-{}]", bar_start, bar_end),
                    position: Point::new(start_x + 7.0, 2.0),
                    color: Color::from_rgb8(0x20, 0x15, 0x05),
                    size: 8.0.into(),
                    ..Default::default()
                });
            }
        }

        // 5b. Ruler Bar Numbers and Precision Sub-Beat Hash Marks
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

                    // Bar Number Text with Sub-pixel Baseline
                    frame.fill_text(canvas::Text {
                        content: format!("{}", bar_num),
                        position: Point::new(x + 3.0, 14.0),
                        color: Color::from_rgb8(0xd1, 0xd5, 0xdb),
                        size: 10.0.into(),
                        ..Default::default()
                    });
                } else if ruler_tick.is_multiple_of(PPQN) {
                    // Beat Hash Mark
                    let tick_line = Path::line(Point::new(x, 20.0), Point::new(x, RULER_HEIGHT));
                    frame.stroke(
                        &tick_line,
                        Stroke::default()
                            .with_color(Color::from_rgb8(0x8a, 0x90, 0xa0))
                            .with_width(0.9),
                    );
                } else if self.horizontal_zoom >= 1.5 && ruler_tick.is_multiple_of(PPQN / 2) {
                    // 8th Note Sub-tick
                    let tick_line = Path::line(Point::new(x, 24.0), Point::new(x, RULER_HEIGHT));
                    frame.stroke(
                        &tick_line,
                        Stroke::default()
                            .with_color(Color::from_rgba8(0x6b, 0x72, 0x80, 0.70))
                            .with_width(0.7),
                    );
                }
            }
            ruler_tick += 240; // 16th Note resolution
        }

        // 6. Logic Pro / FL Studio Iconic Glowing Laser Playhead
        let playhead_x = self.tick_to_x(self.playhead_tick, bounds.width);
        let accent_glow = if self.is_fl_theme {
            Color::from_rgba8(0xf8, 0x8a, 0x36, 0.40)
        } else {
            Color::from_rgba8(0x00, 0xa8, 0xff, 0.40)
        };

        // 6a. Playhead Downward Jewel-Cut Arrow Head in Ruler with Drop Shadow
        let shadow_head = Path::new(|p| {
            p.move_to(Point::new(playhead_x - 6.0, 3.0));
            p.line_to(Point::new(playhead_x + 6.0, 3.0));
            p.line_to(Point::new(playhead_x, RULER_HEIGHT - 1.0));
            p.close();
        });
        frame.fill(&shadow_head, Color::from_rgba8(0, 0, 0, 0.50));

        let triangle_head = Path::new(|p| {
            p.move_to(Point::new(playhead_x - 6.0, 2.0));
            p.line_to(Point::new(playhead_x + 6.0, 2.0));
            p.line_to(Point::new(playhead_x, RULER_HEIGHT - 2.0));
            p.close();
        });
        frame.fill(&triangle_head, Color::WHITE);

        // Center jewel pip inside triangle
        let pip = Path::circle(Point::new(playhead_x, 8.0), 1.5);
        frame.fill(
            &pip,
            if self.is_fl_theme {
                Color::from_rgb8(0xf8, 0x8a, 0x36)
            } else {
                Color::from_rgb8(0x00, 0x7a, 0xff)
            },
        );

        // 6b. Multi-Layer Playhead Laser Guide Line extending across all lanes
        // Outer soft halo wash (6px)
        let halo_line = Path::line(
            Point::new(playhead_x, RULER_HEIGHT),
            Point::new(playhead_x, bounds.height),
        );
        frame.stroke(
            &halo_line,
            Stroke::default()
                .with_color(Color::from_rgba8(255, 255, 255, 0.08))
                .with_width(6.0),
        );

        // Mid neon aura (3px)
        frame.stroke(
            &halo_line,
            Stroke::default().with_color(accent_glow).with_width(3.0),
        );

        // Razor-sharp core laser needle (1.2px pure white)
        frame.stroke(
            &halo_line,
            Stroke::default().with_color(Color::WHITE).with_width(1.2),
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
