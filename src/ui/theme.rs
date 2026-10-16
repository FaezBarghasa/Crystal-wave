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
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Graphite => "Logic Pro Graphite",
            Self::FlDarkOrange => "FL Studio Dark Orange",
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

    /// FL Studio 2025 Dark Orange Theme (.flstheme inspired)
    pub fn fl_dark_orange() -> Self {
        Self {
            preset: ThemePreset::FlDarkOrange,
            background: Color::from_rgb8(0x15, 0x15, 0x18),
            surface: Color::from_rgb8(0x1e, 0x1e, 0x24),
            surface_elevated: Color::from_rgb8(0x28, 0x28, 0x32),
            surface_sunken: Color::from_rgb8(0x11, 0x11, 0x14),
            border: Color::from_rgb8(0x38, 0x38, 0x44),
            border_focus: Color::from_rgb8(0xf8, 0x8a, 0x36), // FL Iconic Orange
            text_primary: Color::from_rgb8(0xf4, 0xf4, 0xf6),
            text_secondary: Color::from_rgb8(0xbf, 0xc0, 0xca),
            text_muted: Color::from_rgb8(0x6e, 0x70, 0x7c),
            accent: Color::from_rgb8(0xf8, 0x8a, 0x36), // FL Vibrant Orange
            accent_active: Color::from_rgb8(0xff, 0xa7, 0x60),
            selected: Color::from_rgb8(0xf8, 0x8a, 0x36),
            highlight: Color::from_rgb8(0x49, 0x77, 0xf6), // FL Secondary Highlight Blue
            mute: Color::from_rgb8(0x4a, 0x36, 0x28),
            solo: Color::from_rgb8(0xf3, 0x9c, 0x12),
            record: Color::from_rgb8(0xd6, 0x30, 0x31),
            step_even: Color::from_rgb8(0x52, 0x55, 0x65), // FL StepEven
            step_odd: Color::from_rgb8(0x32, 0x34, 0x40),  // FL StepOdd
            meters: [
                Color::from_rgb8(0x0e, 0x6a, 0x38),
                Color::from_rgb8(0x1e, 0xbb, 0x5d),
                Color::from_rgb8(0x8e, 0xd0, 0x38),
                Color::from_rgb8(0xf3, 0x9c, 0x12),
                Color::from_rgb8(0xe6, 0x7e, 0x22),
                Color::from_rgb8(0xd6, 0x30, 0x31),
            ],
            note_colors: [
                Color::from_rgb8(0x49, 0x77, 0xf6), // 0: FL Note 0
                Color::from_rgb8(0xf8, 0x8a, 0x36), // 1: FL Orange
                Color::from_rgb8(0x2e, 0xcc, 0x71), // 2: Lime Green
                Color::from_rgb8(0x9b, 0x59, 0xb6), // 3: Purple
                Color::from_rgb8(0x1a, 0xbc, 0x9c), // 4: Turquoise
                Color::from_rgb8(0xe6, 0x7e, 0x22), // 5: Dark Orange
                Color::from_rgb8(0x34, 0x98, 0xdb), // 6: Blue
                Color::from_rgb8(0xe7, 0x4c, 0x3c), // 7: Red
                Color::from_rgb8(0xf1, 0xc4, 0x0f), // 8: Yellow
                Color::from_rgb8(0x16, 0xa0, 0x85), // 9: Green Sea
                Color::from_rgb8(0x8e, 0x44, 0xad), // 10: Wisteria
                Color::from_rgb8(0xd3, 0x54, 0x00), // 11: Pumpkin
                Color::from_rgb8(0x29, 0x80, 0xb9), // 12: Belize Hole
                Color::from_rgb8(0xc0, 0x39, 0x2b), // 13: Pomegranate
                Color::from_rgb8(0xbd, 0xc3, 0xc7), // 14: Silver
                Color::from_rgb8(0x7f, 0x8c, 0x8d), // 15: Asbestos
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

    /// Luxury Vacuum Fluorescent Display (VFD) / OLED instrument panel container
    pub fn oled_vfd_display_style(&self) -> container::Style {
        let border_glow = match self.preset {
            ThemePreset::Graphite => Color::from_rgba8(0x00, 0xa8, 0xff, 0.35),
            ThemePreset::FlDarkOrange => Color::from_rgba8(0xf8, 0x8a, 0x36, 0.40),
        };
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x08, 0x09, 0x0d))),
            text_color: Some(match self.preset {
                ThemePreset::Graphite => Color::from_rgb8(0x5a, 0xe0, 0xff), // VFD Cyan
                ThemePreset::FlDarkOrange => Color::from_rgb8(0xff, 0xb8, 0x5a), // FL Amber VFD
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

    /// Tactile 16-step sequencer button style with illuminated LED core
    pub fn step_pad_style<'a>(
        &'a self,
        active: bool,
        is_even_beat: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
        move |_theme, status| {
            let (bg, border_col) = if active {
                let col = self.accent;
                match status {
                    button::Status::Hovered => (
                        Color {
                            r: (col.r * 1.2).min(1.0),
                            g: (col.g * 1.2).min(1.0),
                            b: (col.b * 1.2).min(1.0),
                            a: 1.0,
                        },
                        Color::WHITE,
                    ),
                    _ => (col, self.accent_active),
                }
            } else {
                let base = if is_even_beat {
                    self.step_even
                } else {
                    self.step_odd
                };
                match status {
                    button::Status::Hovered => (
                        Color {
                            r: (base.r * 1.25).min(1.0),
                            g: (base.g * 1.25).min(1.0),
                            b: (base.b * 1.25).min(1.0),
                            a: base.a,
                        },
                        self.border_focus,
                    ),
                    _ => (base, self.border),
                }
            };

            button::Style {
                background: Some(Background::Color(bg)),
                text_color: if active {
                    Color::WHITE
                } else {
                    self.text_muted
                },
                border: Border {
                    color: border_col,
                    width: if active { 1.5 } else { 1.0 },
                    radius: Radius::from(3.0),
                },
                shadow: Shadow::default(),
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
}

