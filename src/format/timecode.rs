use crate::timeline::PPQN;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmpteFrameRate {
    Fps24,
    Fps25,
    Fps2997,
    Fps30,
    Fps60,
}

impl SmpteFrameRate {
    pub fn as_f32(&self) -> f32 {
        match self {
            SmpteFrameRate::Fps24 => 24.0,
            SmpteFrameRate::Fps25 => 25.0,
            SmpteFrameRate::Fps2997 => 29.97,
            SmpteFrameRate::Fps30 => 30.0,
            SmpteFrameRate::Fps60 => 60.0,
        }
    }
}

/// SMPTE Timecode (HH:MM:SS:FF) representation for exact film/broadcast synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmpteTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub subframes: u8, // 0..99
    pub frame_rate: SmpteFrameRate,
}

impl SmpteTimecode {
    pub fn new(hours: u8, minutes: u8, seconds: u8, frames: u8, rate: SmpteFrameRate) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            frames,
            subframes: 0,
            frame_rate: rate,
        }
    }

    /// Parses a standard timecode string "01:04:22:14"
    pub fn parse(s: &str, rate: SmpteFrameRate) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split(|c| c == ':' || c == ';').collect();
        if parts.len() < 4 {
            return Err("InvalidTimecodeFormat: Must be HH:MM:SS:FF");
        }

        let h: u8 = parts[0].parse().map_err(|_| "InvalidHours")?;
        let m: u8 = parts[1].parse().map_err(|_| "InvalidMinutes")?;
        let sec: u8 = parts[2].parse().map_err(|_| "InvalidSeconds")?;
        let f: u8 = parts[3].parse().map_err(|_| "InvalidFrames")?;

        Ok(Self::new(h, m, sec, f, rate))
    }

    /// Converts timecode into absolute seconds
    pub fn to_seconds(&self) -> f64 {
        let fps = self.frame_rate.as_f32() as f64;
        let total_secs = (self.hours as f64) * 3600.0
            + (self.minutes as f64) * 60.0
            + (self.seconds as f64)
            + ((self.frames as f64) + (self.subframes as f64) / 100.0) / fps;
        total_secs
    }

    /// Converts timecode into musical arrangement ticks at a given tempo (BPM)
    pub fn to_ticks(&self, bpm: f32) -> u64 {
        let seconds = self.to_seconds();
        let ticks_per_second = (PPQN as f64) * (bpm as f64 / 60.0);
        (seconds * ticks_per_second).round() as u64
    }

    /// Formats timecode back to "HH:MM:SS:FF" string
    pub fn to_string_formatted(&self) -> String {
        format!(
            "{:02}:{:02}:{:02}:{:02}",
            self.hours, self.minutes, self.seconds, self.frames
        )
    }
}
