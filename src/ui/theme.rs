//! UI Theme Tokens and Design System
//!
//! Provides tokenized design colors inspired by Apple Logic Pro (Graphite)
//! and Image-Line FL Studio (Dark Orange token schema from .flstheme).

use iced::border::Radius;
use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow};
use serde::{Deserialize, Serialize};

/// Available theme presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemePreset {
    #[default]
    Graphite,
    FlDarkOrange,
    FlStudioModern,
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Graphite => "Logic Pro Graphite",
            Self::FlDarkOrange => "FL Studio Dark Orange",
            Self::FlStudioModern => "FL Studio 2026 Modern",
        }
    }
}

/// Tokenized design system palette
#[derive(Debug, Clone, PartialEq)]
pub struct UiTheme {
    pub preset: ThemePreset,
    pub background: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_sunken: Color,
    pub border: Color,
    pub border_focus: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_active: Color,
    pub selected: Color,
    pub highlight: Color,
    pub mute: Color,
    pub solo: Color,
    pub record: Color,
    pub step_even: Color,
    pub step_odd: Color,
    pub step_active: Color,
    pub step_active_dim: Color,
    pub channel_name_bg: Color,
    pub channel_name_sel: Color,
    pub window_titlebar: Color,
    pub knob_track: Color,
    pub knob_thumb: Color,
    pub grid_background: Color,
    pub meters: [Color; 6],
    pub note_colors: [Color; 16],
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::from_preset(ThemePreset::Graphite)
    }
}

impl UiTheme {
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Graphite => Self::graphite(),
            ThemePreset::FlDarkOrange => Self::fl_dark_orange(),
            ThemePreset::FlStudioModern => Self::fl_studio_modern(),
        }
    }

    /// Apple Logic Pro 12.3.1 Sleek Graphite Dark Theme
    pub fn graphite() -> Self {
        Self {
            preset: ThemePreset::Graphite,
            background: Color::from_rgb8(0x13, 0x14, 0x18),
            surface: Color::from_rgb8(0x1b, 0x1d, 0x22),
            surface_elevated: Color::from_rgb8(0x23, 0x27, 0x30),
            surface_sunken: Color::from_rgb8(0x10, 0x11, 0x14),
            border: Color::from_rgb8(0x2d, 0x31, 0x3c),
            border_focus: Color::from_rgb8(0x00, 0x7a, 0xff),
            text_primary: Color::from_rgb8(0xf0, 0xf3, 0xf8),
            text_secondary: Color::from_rgb8(0xc0, 0xc6, 0xd4),
            text_muted: Color::from_rgb8(0x72, 0x79, 0x8a),
            accent: Color::from_rgb8(0x00, 0x7a, 0xff), // Apple Blue
            accent_active: Color::from_rgb8(0x33, 0x95, 0xff),
            selected: Color::from_rgb8(0x00, 0x7a, 0xff),
            highlight: Color::from_rgb8(0x22, 0x66, 0xbb),
            mute: Color::from_rgb8(0x3a, 0x3d, 0x47),
            solo: Color::from_rgb8(0xe5, 0xa9, 0x3c), // Logic Amber
            record: Color::from_rgb8(0xe0, 0x31, 0x31), // Crimson Red
            step_even: Color::from_rgb8(0x34, 0x3a, 0x46),
            step_odd: Color::from_rgb8(0x25, 0x2a, 0x33),
            step_active: Color::from_rgb8(0x00, 0x7a, 0xff),
            step_active_dim: Color::from_rgb8(0x00, 0x55, 0xb3),
            channel_name_bg: Color::from_rgb8(0x28, 0x2c, 0x37),
            channel_name_sel: Color::from_rgb8(0x36, 0x3c, 0x4a),
            window_titlebar: Color::from_rgb8(0x1b, 0x1d, 0x24),
            knob_track: Color::from_rgb8(0x10, 0x11, 0x14),
            knob_thumb: Color::from_rgb8(0xc8, 0xca, 0xd0),
            grid_background: Color::from_rgb8(0x14, 0x16, 0x1b),
            meters: [
                Color::from_rgb8(0x1e, 0x82, 0x4c), // -48 dB Dark Green
                Color::from_rgb8(0x2e, 0xcc, 0x71), // -18 dB Emerald Green
                Color::from_rgb8(0xa3, 0xe0, 0x48), // -12 dB Lime
                Color::from_rgb8(0xf1, 0xc4, 0x0f), // -6 dB Yellow
                Color::from_rgb8(0xe6, 0x7e, 0x22), // -2 dB Orange
                Color::from_rgb8(0xe7, 0x4c, 0x3c), // 0 dB / Clip Red
            ],
            note_colors: [
                Color::from_rgb8(0x00, 0x7a, 0xff), // 0: Lead Blue
                Color::from_rgb8(0x1e, 0xa0, 0x50), // 1: Emerald
                Color::from_rgb8(0x8a, 0x42, 0xdb), // 2: Violet
                Color::from_rgb8(0xe6, 0x7e, 0x22), // 3: Amber Orange
                Color::from_rgb8(0x29, 0xb6, 0xf6), // 4: Cyan
                Color::from_rgb8(0x66, 0xbb, 0x6a), // 5: Light Green
                Color::from_rgb8(0xab, 0x47, 0xbc), // 6: Purple
                Color::from_rgb8(0xff, 0xa7, 0x26), // 7: Warm Amber
                Color::from_rgb8(0x26, 0xc6, 0xda), // 8: Teal
                Color::from_rgb8(0x9c, 0xcc, 0x65), // 9: Olive
                Color::from_rgb8(0xec, 0x40, 0x7a), // 10: Rose Pink
                Color::from_rgb8(0xff, 0xca, 0x28), // 11: Golden
                Color::from_rgb8(0x42, 0xa5, 0xf5), // 12: Sky Blue
                Color::from_rgb8(0x26, 0xa6, 0x9a), // 13: Mint
                Color::from_rgb8(0x7e, 0x57, 0xc2), // 14: Deep Purple
                Color::from_rgb8(0xff, 0x70, 0x43), // 15: Deep Orange
            ],
        }
    }

    /// FL Studio 2025 Dark Orange Theme (Exact .flstheme Delphi TColor calibration)
    pub fn fl_dark_orange() -> Self {
        Self {
            preset: ThemePreset::FlDarkOrange,
            background: Color::from_rgb8(0x1a, 0x1a, 0x1f),
            surface: Color::from_rgb8(0x23, 0x23, 0x29),
            surface_elevated: Color::from_rgb8(0x2d, 0x2d, 0x35),
            surface_sunken: Color::from_rgb8(0x0f, 0x0f, 0x12),
            border: Color::from_rgb8(0x3e, 0x3e, 0x4a),
            border_focus: Color::from_rgb8(0xf8, 0x8a, 0x36), // FL Iconic Orange
            text_primary: Color::from_rgb8(209, 207, 206),    // TextColor: -3223599 (#d1cfce)
            text_secondary: Color::from_rgb8(0xbf, 0xc0, 0xca),
            text_muted: Color::from_rgb8(0x6e, 0x70, 0x7c),
            accent: Color::from_rgb8(0xf8, 0x8a, 0x36), // FL Vibrant Orange
            accent_active: Color::from_rgb8(0xff, 0xa7, 0x60),
            selected: Color::from_rgb8(54, 138, 248), // Selected: -488906 (#368af8)
            highlight: Color::from_rgb8(246, 119, 73), // Highlight: -11962378 (#f67749)
            mute: Color::from_rgb8(200, 52, 16),      // Mute: -15715128 (#c83410)
            solo: Color::from_rgb8(0xf3, 0x9c, 0x12),
            record: Color::from_rgb8(0xd6, 0x30, 0x31),
            step_even: Color::from_rgb8(0x7a, 0x56, 0x50), // Copper/Brown rubber pad (Beats 2 & 4)
            step_odd: Color::from_rgb8(0xb0, 0xb4, 0xbe),  // Silver/Grey rubber pad (Beats 1 & 3)
            step_active: Color::from_rgb8(0xff, 0x6b, 0x81), // FL StepActive Rose Pink
            step_active_dim: Color::from_rgb8(0xd6, 0x55, 0x68),
            channel_name_bg: Color::from_rgb8(0x35, 0x38, 0x40), // FL Channel Name Pill
            channel_name_sel: Color::from_rgb8(0x41, 0x44, 0x50), // FL Selected Channel
            window_titlebar: Color::from_rgb8(0x20, 0x22, 0x28), // Sub-window header
            knob_track: Color::from_rgb8(0x11, 0x13, 0x18),
            knob_thumb: Color::from_rgb8(0xc8, 0xca, 0xd0),
            grid_background: Color::from_rgb8(52, 68, 78), // PRGridback / PLGridback: 5129268 (#34444e)
            meters: [
                Color::from_rgb8(170, 57, 11),   // Meter0: -16041558 (#aa390b)
                Color::from_rgb8(248, 119, 36),  // Meter1: -14387208 (#f87724)
                Color::from_rgb8(246, 161, 3),   // Meter2: -16539146 (#f6a103)
                Color::from_rgb8(238, 197, 78),  // Meter3: -11614738 (#eec54e)
                Color::from_rgb8(255, 255, 255), // Meter4: -1 (#ffffff)
                Color::from_rgb8(255, 255, 255), // Meter5: -1 (#ffffff)
            ],
            note_colors: [
                Color::from_rgb8(241, 68, 3),    // NoteColor0: -16562959 (#f14403)
                Color::from_rgb8(229, 99, 58),   // NoteColor1: -12950555 (#e5633a)
                Color::from_rgb8(206, 121, 93),  // NoteColor2: -10651186 (#ce795d)
                Color::from_rgb8(197, 163, 154), // NoteColor3: -6642747 (#c5a39a)
                Color::from_rgb8(255, 165, 5),   // NoteColor4: -16407041 (#ffa505)
                Color::from_rgb8(231, 167, 32),  // NoteColor5: -14637081 (#e7a720)
                Color::from_rgb8(196, 147, 71),  // NoteColor6: -12086332 (#c49347)
                Color::from_rgb8(177, 143, 90),  // NoteColor7: -10842191 (#b18f5a)
                Color::from_rgb8(183, 225, 0),   // NoteColor8: -16719433 (#b7e100)
                Color::from_rgb8(168, 207, 34),  // NoteColor9: -14495832 (#a8cf22)
                Color::from_rgb8(125, 175, 68),  // NoteColor10: -12275843 (#7daf44)
                Color::from_rgb8(92, 136, 65),   // NoteColor11: -12482468 (#5c8841)
                Color::from_rgb8(0, 222, 55),    // NoteColor12: -13115904 (#00de37)
                Color::from_rgb8(49, 209, 94),   // NoteColor13: -10563279 (#31d15e)
                Color::from_rgb8(86, 177, 119),  // NoteColor14: -8933034 (#56b177)
                Color::from_rgb8(140, 227, 139), // NoteColor15: -7609460 (#8ce38b)
            ],
        }
    }

    /// FL Studio 2026 Modern OLED Theme (Deep charcoal with radiant neon citrus orange)
    pub fn fl_studio_modern() -> Self {
        Self {
            preset: ThemePreset::FlStudioModern,
            background: Color::from_rgb8(0x10, 0x11, 0x14), // Deep OLED Charcoal
            surface: Color::from_rgb8(0x18, 0x1a, 0x20),    // Slate Surface
            surface_elevated: Color::from_rgb8(0x22, 0x25, 0x2d),
            surface_sunken: Color::from_rgb8(0x0a, 0x0b, 0x0d), // Sunken Well
            border: Color::from_rgb8(0x2c, 0x30, 0x3a),
            border_focus: Color::from_rgb8(0xff, 0x88, 0x00), // Citrus Orange
            text_primary: Color::from_rgb8(0xf4, 0xf6, 0xfa),
            text_secondary: Color::from_rgb8(0xc2, 0xc7, 0xd4),
            text_muted: Color::from_rgb8(0x70, 0x76, 0x88),
            accent: Color::from_rgb8(0xff, 0x88, 0x00), // Citrus Accent
            accent_active: Color::from_rgb8(0xff, 0xa5, 0x33),
            selected: Color::from_rgb8(0xff, 0x88, 0x00),
            highlight: Color::from_rgb8(0x00, 0xf2, 0xfe), // Electric Cyan
            mute: Color::from_rgb8(0x3d, 0x2e, 0x22),
            solo: Color::from_rgb8(0xf5, 0xa6, 0x23),
            record: Color::from_rgb8(0xe7, 0x4c, 0x3c),
            step_even: Color::from_rgb8(0x7a, 0x56, 0x50), // Copper/Brown rubber pad (Beats 2 & 4)
            step_odd: Color::from_rgb8(0xb0, 0xb4, 0xbe),  // Silver/Grey rubber pad (Beats 1 & 3)
            step_active: Color::from_rgb8(0xff, 0x6b, 0x81), // FL StepActive Rose Pink
            step_active_dim: Color::from_rgb8(0xd6, 0x55, 0x68),
            channel_name_bg: Color::from_rgb8(0x28, 0x2c, 0x34),
            channel_name_sel: Color::from_rgb8(0x38, 0x3c, 0x46),
            window_titlebar: Color::from_rgb8(0x19, 0x1b, 0x22),
            knob_track: Color::from_rgb8(0x0e, 0x10, 0x14),
            knob_thumb: Color::from_rgb8(0xd0, 0xd4, 0xdc),
            grid_background: Color::from_rgb8(0x18, 0x1a, 0x20),
            meters: [
                Color::from_rgb8(0x0e, 0x6a, 0x38), // -48 dB Dark Green
                Color::from_rgb8(0x1e, 0xbb, 0x5d), // -18 dB Emerald Green
                Color::from_rgb8(0x38, 0xef, 0x7d), // -12 dB Neon Lime
                Color::from_rgb8(0xf3, 0x9c, 0x12), // -6 dB Yellow
                Color::from_rgb8(0xff, 0x88, 0x00), // -2 dB Citrus Orange
                Color::from_rgb8(0xff, 0x2d, 0x55), // 0 dB / Clip Neon Red
            ],
            note_colors: [
                Color::from_rgb8(0x49, 0x77, 0xf6), // 0: Sky Blue
                Color::from_rgb8(0xff, 0x88, 0x00), // 1: Citrus Orange
                Color::from_rgb8(0x38, 0xef, 0x7d), // 2: Neon Lime
                Color::from_rgb8(0x9b, 0x59, 0xb6), // 3: Purple
                Color::from_rgb8(0x00, 0xf2, 0xfe), // 4: Electric Cyan
                Color::from_rgb8(0xe6, 0x7e, 0x22), // 5: Amber
                Color::from_rgb8(0x34, 0x98, 0xdb), // 6: Deep Blue
                Color::from_rgb8(0xe7, 0x4c, 0x3c), // 7: Red
                Color::from_rgb8(0xf1, 0xc4, 0x0f), // 8: Yellow
                Color::from_rgb8(0x16, 0xa0, 0x85), // 9: Teal
                Color::from_rgb8(0xa8, 0x55, 0xf7), // 10: Violet
                Color::from_rgb8(0xf9, 0x73, 0x16), // 11: Warm Orange
                Color::from_rgb8(0x06, 0xb6, 0xd4), // 12: Cyan
                Color::from_rgb8(0x10, 0xb9, 0x81), // 13: Emerald
                Color::from_rgb8(0xec, 0x48, 0x99), // 14: Pink
                Color::from_rgb8(0x64, 0x74, 0x8b), // 15: Slate
            ],
        }
    }

    /// Get color for note index (0..15)
    pub fn note_color(&self, index: usize) -> Color {
        self.note_colors[index % self.note_colors.len()]
    }

    // --- Styling helpers for Iced widgets ---

    /// Card / Panel container style
    pub fn panel_container_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.surface)),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border,
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Header / Toolbar container style
    pub fn header_container_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.surface_elevated)),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border,
                width: 1.0,
                radius: Radius::from(0.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Sunken display (LCD / meters) container style
    pub fn sunken_container_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.surface_sunken)),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border,
                width: 1.0,
                radius: Radius::from(3.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Standard DAW button style
    pub fn button_style<'a>(
        &'a self,
        active: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let bg = if active {
                self.accent
            } else {
                match status {
                    button::Status::Active => self.surface_elevated,
                    button::Status::Hovered => Color {
                        r: (self.surface_elevated.r * 1.2).min(1.0),
                        g: (self.surface_elevated.g * 1.2).min(1.0),
                        b: (self.surface_elevated.b * 1.2).min(1.0),
                        a: self.surface_elevated.a,
                    },
                    button::Status::Pressed => self.surface,
                    button::Status::Disabled => self.surface_sunken,
                }
            };
            let text_col = if active {
                Color::WHITE
            } else {
                match status {
                    button::Status::Disabled => self.text_muted,
                    _ => self.text_primary,
                }
            };
            let border_col = if active {
                self.accent_active
            } else {
                self.border
            };

            button::Style {
                background: Some(Background::Color(bg)),
                text_color: text_col,
                border: Border {
                    color: border_col,
                    width: 1.0,
                    radius: Radius::from(3.0),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
    }

    /// Accent button style (Play, Bounce, Record)
    pub fn accent_button_style<'a>(
        &'a self,
        color: Color,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let bg = match status {
                button::Status::Active => color,
                button::Status::Hovered => Color {
                    r: (color.r * 1.15).min(1.0),
                    g: (color.g * 1.15).min(1.0),
                    b: (color.b * 1.15).min(1.0),
                    a: color.a,
                },
                button::Status::Pressed => Color {
                    r: color.r * 0.85,
                    g: color.g * 0.85,
                    b: color.b * 0.85,
                    a: color.a,
                },
                button::Status::Disabled => self.surface_sunken,
            };

            button::Style {
                background: Some(Background::Color(bg)),
                text_color: Color::WHITE,
                border: Border {
                    color: self.border,
                    width: 1.0,
                    radius: Radius::from(3.0),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
    }

    /// Toggleable accent button style (inactive surface vs active illuminated color)
    pub fn toggle_accent_button_style<'a>(
        &'a self,
        active: bool,
        color: Color,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            if active {
                (self.accent_button_style(color))(_theme, status)
            } else {
                (self.button_style(false))(_theme, status)
            }
        }
    }

    /// Luxury Vacuum Fluorescent Display (VFD) / OLED instrument panel container
    pub fn oled_vfd_display_style(&self) -> container::Style {
        let border_glow = match self.preset {
            ThemePreset::Graphite => Color::from_rgba8(0x00, 0xa8, 0xff, 0.35),
            ThemePreset::FlDarkOrange => Color::from_rgba8(0xf8, 0x8a, 0x36, 0.40),
            ThemePreset::FlStudioModern => Color::from_rgba8(0xff, 0x88, 0x00, 0.50),
        };
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x08, 0x09, 0x0d))),
            text_color: Some(match self.preset {
                ThemePreset::Graphite => Color::from_rgb8(0x5a, 0xe0, 0xff), // VFD Cyan
                ThemePreset::FlDarkOrange => Color::from_rgb8(0xff, 0xb8, 0x5a), // FL Amber VFD
                ThemePreset::FlStudioModern => Color::from_rgb8(0xff, 0xaa, 0x44), // FL Modern Amber Neon
            }),
            border: Border {
                color: border_glow,
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Double-bezel outer frame container style
    pub fn outer_bezel_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.surface_elevated)),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border,
                width: 1.5,
                radius: Radius::from(6.0),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            ..container::Style::default()
        }
    }

    /// Machined sunken inner well container style
    pub fn inner_well_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.surface_sunken)),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border,
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Machined capsule container style for modular toolbar segments
    pub fn machined_capsule_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.surface)),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border,
                width: 1.0,
                radius: Radius::from(6.0),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            },
            ..container::Style::default()
        }
    }

    /// Studio hardware rack panel container with beveled chamfer
    pub fn hardware_rack_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.surface)),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border,
                width: 1.0,
                radius: Radius::from(2.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Tactile 16-step sequencer button style with authentic FL Studio 3D convex rubber pad and LED core
    pub fn step_pad_style<'a>(
        &'a self,
        active: bool,
        is_even_beat: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let base = match (active, is_even_beat) {
                (true, true) => self.step_active,
                (true, false) => self.step_active_dim,
                (false, true) => self.step_even,
                (false, false) => self.step_odd,
            };
            let bg = match status {
                button::Status::Hovered => Color {
                    r: (base.r * 1.18).min(1.0),
                    g: (base.g * 1.18).min(1.0),
                    b: (base.b * 1.18).min(1.0),
                    a: base.a,
                },
                button::Status::Pressed => Color {
                    r: base.r * 0.80,
                    g: base.g * 0.80,
                    b: base.b * 0.80,
                    a: base.a,
                },
                _ => base,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: if active {
                    Color::from_rgb8(0xff, 0xff, 0xff)
                } else {
                    Color::from_rgba8(255, 255, 255, 0.25)
                },
                border: Border {
                    color: if active {
                        Color::from_rgba8(255, 255, 255, 0.65) // Glowing core highlight
                    } else if is_even_beat {
                        Color::from_rgba8(255, 255, 255, 0.14) // Top bevel highlight
                    } else {
                        Color::from_rgba8(0, 0, 0, 0.45) // Sunken groove border
                    },
                    width: if active { 1.0 } else { 0.75 },
                    radius: Radius::from(3.0),
                },
                shadow: if active {
                    Shadow {
                        color: Color::from_rgba8(212, 105, 123, 0.40), // Radiant rose LED glow
                        offset: iced::Vector::new(0.0, 0.0),
                        blur_radius: 4.0,
                    }
                } else {
                    Shadow {
                        color: Color::from_rgba8(0, 0, 0, 0.35),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 1.0,
                    }
                },
                ..button::Style::default()
            }
        }
    }

    /// Neve / SSL Console-Grade multi-segment meter color mapping for dBFS
    pub fn meter_color_for_db(&self, db: f32) -> Color {
        if db >= 0.0 {
            self.meters[5] // Clip Red
        } else if db >= -3.0 {
            self.meters[4] // Orange
        } else if db >= -6.0 {
            self.meters[3] // Yellow
        } else if db >= -12.0 {
            self.meters[2] // Lime
        } else if db >= -24.0 {
            self.meters[1] // Emerald
        } else {
            self.meters[0] // Dark Green
        }
    }

    /// Curated luxury track accent color for stem identification
    pub fn track_accent_color(&self, index: usize) -> Color {
        const TRACK_PALETTE: [Color; 8] = [
            Color::from_rgb(0.00, 0.48, 1.00), // Sapphire Blue (Vocals)
            Color::from_rgb(0.96, 0.42, 0.20), // Coral Orange (Drums)
            Color::from_rgb(0.12, 0.78, 0.45), // Emerald Green (Bass)
            Color::from_rgb(0.62, 0.28, 0.90), // Royal Amethyst (Synths/Keys)
            Color::from_rgb(0.95, 0.72, 0.15), // Amber Gold (Lead / Guitar)
            Color::from_rgb(0.15, 0.75, 0.85), // Electric Cyan (Percussion)
            Color::from_rgb(0.92, 0.25, 0.45), // Ruby Pink (Effects / FX)
            Color::from_rgb(0.55, 0.60, 0.70), // Platinum Slate (Aux / Bus)
        ];
        TRACK_PALETTE[index % TRACK_PALETTE.len()]
    }

    /// Authentic FL Studio amber neon readout container style
    pub fn vfd_amber_display_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x0a, 0x09, 0x07))),
            text_color: Some(Color::from_rgb8(0xff, 0xb8, 0x33)),
            border: Border {
                color: Color::from_rgba8(0xff, 0x88, 0x00, 0.45),
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Electric cyan telemetry display container style
    pub fn vfd_cyan_display_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x06, 0x09, 0x0d))),
            text_color: Some(Color::from_rgb8(0x5a, 0xe0, 0xff)),
            border: Border {
                color: Color::from_rgba8(0x00, 0xa8, 0xff, 0.40),
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Transport pill button style for PAT / SONG and cycle
    pub fn transport_pill_style<'a>(
        &'a self,
        active: bool,
        accent: Color,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let bg = if active {
                accent
            } else {
                match status {
                    button::Status::Hovered => Color {
                        r: (self.surface_elevated.r * 1.25).min(1.0),
                        g: (self.surface_elevated.g * 1.25).min(1.0),
                        b: (self.surface_elevated.b * 1.25).min(1.0),
                        a: self.surface_elevated.a,
                    },
                    button::Status::Pressed => self.surface,
                    _ => self.surface_elevated,
                }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: if active {
                    Color::BLACK
                } else {
                    self.text_primary
                },
                border: Border {
                    color: if active { Color::WHITE } else { self.border },
                    width: if active { 1.5 } else { 1.0 },
                    radius: Radius::from(10.0),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
    }

    /// LED status jewel dot container style (Mute/Solo green, Record red, Windows amber)
    pub fn led_jewel_style(&self, active: bool, color: Color) -> container::Style {
        let bg = if active {
            color
        } else {
            Color::from_rgba(color.r * 0.25, color.g * 0.25, color.b * 0.25, 0.4)
        };
        let border_col = if active {
            Color::from_rgba(color.r, color.g, color.b, 0.8)
        } else {
            Color::from_rgba(0.2, 0.2, 0.2, 0.5)
        };
        container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border_col,
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Channel row button style with left accent highlight
    pub fn fl_channel_button_style<'a>(
        &'a self,
        selected: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let bg = if selected {
                Color::from_rgb8(0x28, 0x2d, 0x38)
            } else {
                match status {
                    button::Status::Hovered => Color::from_rgb8(0x24, 0x27, 0x30),
                    button::Status::Pressed => Color::from_rgb8(0x18, 0x1a, 0x20),
                    _ => Color::from_rgb8(0x1e, 0x20, 0x27),
                }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: if selected {
                    Color::WHITE
                } else {
                    self.text_primary
                },
                border: Border {
                    color: if selected {
                        self.accent
                    } else {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                    },
                    width: if selected { 1.5 } else { 1.0 },
                    radius: Radius::from(3.0),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
    }

    /// Mixer channel strip container style
    pub fn mixer_strip_style(&self, is_master: bool, is_selected: bool) -> container::Style {
        let bg = if is_master {
            Color::from_rgb8(0x16, 0x19, 0x22)
        } else if is_selected {
            Color::from_rgb8(0x20, 0x24, 0x2e)
        } else {
            self.surface
        };
        let border_col = if is_selected {
            self.accent
        } else if is_master {
            Color::from_rgba8(0x5a, 0xe0, 0xff, 0.4)
        } else {
            self.border
        };
        container::Style {
            background: Some(Background::Color(bg)),
            text_color: Some(self.text_primary),
            border: Border {
                color: border_col,
                width: if is_selected { 1.5 } else { 1.0 },
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Recessed dark well for mini controls / knobs
    pub fn knob_well_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x0c, 0x0d, 0x11))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 1.0,
                radius: Radius::from(3.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// FL Studio 2026 Top Modular Bar Container Style
    pub fn fl_top_bar_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x13, 0x15, 0x1b))),
            text_color: Some(self.text_primary),
            border: Border {
                color: Color::from_rgba8(255, 255, 255, 0.08),
                width: 1.0,
                radius: Radius::from(0.0),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..container::Style::default()
        }
    }

    /// FPC (Fruity Pad Controller) 4x4 drum pad style
    pub fn fl_pad_style<'a>(
        &'a self,
        base_color: Color,
        is_triggered: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let bg = if is_triggered {
                Color::WHITE
            } else {
                match status {
                    button::Status::Hovered => Color {
                        r: (base_color.r * 1.25).min(1.0),
                        g: (base_color.g * 1.25).min(1.0),
                        b: (base_color.b * 1.25).min(1.0),
                        a: 1.0,
                    },
                    button::Status::Pressed => Color::WHITE,
                    _ => Color {
                        r: base_color.r * 0.75,
                        g: base_color.g * 0.75,
                        b: base_color.b * 0.75,
                        a: 0.95,
                    },
                }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: if is_triggered {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
                border: Border {
                    color: if is_triggered {
                        self.accent
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.6)
                    },
                    width: if is_triggered { 2.0 } else { 1.0 },
                    radius: Radius::from(4.0),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
    }

    /// Insert FX Rack Slot style
    pub fn fl_fx_slot_style(&self, active: bool) -> container::Style {
        container::Style {
            background: Some(Background::Color(if active {
                Color::from_rgb8(0x18, 0x1b, 0x22)
            } else {
                Color::from_rgb8(0x12, 0x13, 0x17)
            })),
            text_color: Some(if active {
                Color::from_rgb8(0xe4, 0xe8, 0xf0)
            } else {
                Color::from_rgb8(0x72, 0x78, 0x86)
            }),
            border: Border {
                color: if active {
                    Color::from_rgba8(0x38, 0xef, 0x7d, 0.25)
                } else {
                    Color::from_rgba8(255, 255, 255, 0.05)
                },
                width: 1.0,
                radius: Radius::from(3.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Sub-window header / titlebar style
    pub fn window_titlebar_style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.window_titlebar)),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border,
                width: 1.0,
                radius: Radius::from(0.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }

    /// Channel name pill button style
    pub fn channel_pill_style<'a>(
        &'a self,
        selected: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let base = if selected {
                self.channel_name_sel
            } else {
                self.channel_name_bg
            };
            let bg = match status {
                button::Status::Hovered => Color {
                    r: (base.r * 1.15).min(1.0),
                    g: (base.g * 1.15).min(1.0),
                    b: (base.b * 1.15).min(1.0),
                    a: base.a,
                },
                button::Status::Pressed => Color {
                    r: base.r * 0.85,
                    g: base.g * 0.85,
                    b: base.b * 0.85,
                    a: base.a,
                },
                _ => base,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: if selected {
                    Color::WHITE
                } else {
                    self.text_primary
                },
                border: Border {
                    color: if selected { self.accent } else { self.border },
                    width: 1.0,
                    radius: Radius::from(4.0),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
    }

    /// Circular transport button style (Play, Stop, Record, Loop)
    pub fn circular_transport_btn_style<'a>(
        &'a self,
        face_color: Color,
        active: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let bg = if active {
                face_color
            } else {
                match status {
                    button::Status::Hovered => Color {
                        r: (self.surface_elevated.r * 1.25).min(1.0),
                        g: (self.surface_elevated.g * 1.25).min(1.0),
                        b: (self.surface_elevated.b * 1.25).min(1.0),
                        a: self.surface_elevated.a,
                    },
                    button::Status::Pressed => self.surface_sunken,
                    _ => self.surface_elevated,
                }
            };
            let text_color = if active {
                Color::WHITE
            } else {
                self.text_secondary
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color,
                border: Border {
                    color: if active { face_color } else { self.border },
                    width: 1.0,
                    radius: Radius::from(14.0),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
    }

    /// F-Key Toolbar Button style with status jewel LED
    pub fn fl_fkey_button_style<'a>(
        &'a self,
        active: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let bg = if active {
                Color::from_rgb8(0x22, 0x27, 0x33)
            } else {
                match status {
                    button::Status::Hovered => Color::from_rgb8(0x1c, 0x1e, 0x26),
                    button::Status::Pressed => Color::from_rgb8(0x10, 0x12, 0x16),
                    _ => Color::from_rgb8(0x15, 0x17, 0x1d),
                }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: if active {
                    Color::WHITE
                } else {
                    self.text_secondary
                },
                border: Border {
                    color: if active {
                        self.accent
                    } else {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                    },
                    width: if active { 1.5 } else { 1.0 },
                    radius: Radius::from(3.0),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
    }
}
