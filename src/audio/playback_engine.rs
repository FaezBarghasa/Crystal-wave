use crate::fl_studio::FlPianoRollNote;
use crate::instruments::virtual_instrument::{AdsrEnvelope, OscillatorWaveform, PolyphonicSynth};
use crate::mixing::MixingEngine;
use crate::timeline::region::{FadeCurve, RegionKind};
use crate::timeline::track::{Track, TrackType};
use std::f32::consts::PI;

pub const PPQN: f64 = 960.0;

/// Sample-accurate timeline playback scheduler and offline audio renderer
pub struct TimelineAudioScheduler {
    pub sample_rate: f32,
    pub tempo_bpm: f32,
    pub swing_amount: f32,
    pub synth: PolyphonicSynth,
    track_buffer_l: Vec<f32>,
    track_buffer_r: Vec<f32>,
}

impl TimelineAudioScheduler {
    pub fn new(sample_rate: f32, tempo_bpm: f32) -> Self {
        let safe_sr = if sample_rate > 8000.0 {
            sample_rate
        } else {
            48000.0
        };
        let safe_bpm = if tempo_bpm > 10.0 { tempo_bpm } else { 120.0 };

        let mut synth = PolyphonicSynth::new("CrystalSynth", 16, safe_sr);
        synth.waveform = OscillatorWaveform::Sawtooth;
        synth.adsr = AdsrEnvelope {
            attack_samples: (safe_sr * 0.015) as usize,
            decay_samples: (safe_sr * 0.08) as usize,
            sustain_level: 0.65,
            release_samples: (safe_sr * 0.15) as usize,
        };

        Self {
            sample_rate: safe_sr,
            tempo_bpm: safe_bpm,
            swing_amount: 0.0,
            synth,
            track_buffer_l: vec![0.0; 2048],
            track_buffer_r: vec![0.0; 2048],
        }
    }

    pub fn set_tempo(&mut self, bpm: f32) {
        if bpm.is_finite() && bpm > 10.0 {
            self.tempo_bpm = bpm;
        }
    }

    pub fn set_swing(&mut self, swing: f32) {
        if swing.is_finite() {
            self.swing_amount = swing.clamp(0.0, 1.0);
        }
    }

    /// Calculate swing-adjusted scheduled tick for 16th-note steps
    pub fn apply_swing_to_tick(&self, tick: u64) -> u64 {
        if self.swing_amount <= 0.0 {
            return tick;
        }
        let step_ticks = 240u64; // 16th note at 960 PPQN
        let step_index = tick / step_ticks;
        if step_index % 2 == 1 {
            // Odd sixteenth note step: delay by swing_amount * 120 ticks
            let delay_ticks = (self.swing_amount * 120.0) as u64;
            tick + delay_ticks
        } else {
            tick
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        if sr.is_finite() && sr > 8000.0 {
            self.sample_rate = sr;
            self.synth.sample_rate = sr;
        }
    }

    /// Convert hardware sample frame count into timeline ticks
    pub fn frame_to_tick(&self, frame: u64) -> u64 {
        let ticks =
            (frame as f64 * PPQN * self.tempo_bpm as f64) / (self.sample_rate as f64 * 60.0);
        ticks as u64
    }

    /// Convert timeline ticks into exact hardware sample frame count
    pub fn tick_to_frame(&self, tick: u64) -> u64 {
        let frames =
            (tick as f64 * self.sample_rate as f64 * 60.0) / (PPQN * self.tempo_bpm as f64);
        frames as u64
    }

    /// Evaluate fade gain multiplier for a frame offset within a region
    fn calculate_fade_multiplier(
        frame_in_reg: u64,
        total_reg_frames: u64,
        fade_in_frames: u64,
        fade_out_frames: u64,
        in_curve: FadeCurve,
        out_curve: FadeCurve,
    ) -> f32 {
        let mut gain = 1.0f32;

        // Fade In
        if fade_in_frames > 0 && frame_in_reg < fade_in_frames {
            let progress = (frame_in_reg as f32 / fade_in_frames as f32).clamp(0.0, 1.0);
            gain *= match in_curve {
                FadeCurve::Linear => progress,
                FadeCurve::EqualPower => (progress * (PI * 0.5)).sin(),
                FadeCurve::Exponential => progress * progress,
                FadeCurve::SCurve => 0.5 * (1.0 - (progress * PI).cos()),
                _ => progress,
            };
        }

        // Fade Out
        if fade_out_frames > 0 && total_reg_frames > fade_out_frames {
            let fade_out_start = total_reg_frames - fade_out_frames;
            if frame_in_reg >= fade_out_start {
                let progress = ((total_reg_frames - frame_in_reg) as f32 / fade_out_frames as f32)
                    .clamp(0.0, 1.0);
                gain *= match out_curve {
                    FadeCurve::Linear => progress,
                    FadeCurve::EqualPower => (progress * (PI * 0.5)).sin(),
                    FadeCurve::Exponential => progress * progress,
                    FadeCurve::SCurve => 0.5 * (1.0 - (progress * PI).cos()),
                    _ => progress,
                };
            }
        }

        gain
    }

    /// Synthesize an organic musical audio sample for an in-memory take region
    fn synthesize_take_sample(
        region_id: usize,
        track_id: usize,
        frame_in_reg: u64,
        sample_rate: f32,
    ) -> (f32, f32) {
        let t = frame_in_reg as f32 / sample_rate;
        // Pitch variations per track / region (harmonic chord progressions)
        let base_freq = match track_id % 4 {
            0 => 220.0, // A3 (Lead / Vocal / Take 1)
            1 => 110.0, // A2 (Bass / Inst 1)
            2 => 330.0, // E4 (Guitar / Pad)
            _ => 165.0, // E3 (Rhythm)
        };

        let f = base_freq * (1.0 + (region_id as f32 * 0.05));
        let env = (-t * 0.8).exp().max(0.15); // gentle decay envelope
        let osc1 = (2.0 * PI * f * t).sin();
        let osc2 = (2.0 * PI * f * 2.0 * t).sin() * 0.35;
        let osc3 = (2.0 * PI * f * 3.0 * t).sin() * 0.15;
        let sample = (osc1 + osc2 + osc3) * 0.28 * env;

        // Subtle stereo chorus / spatial spread
        let left = sample;
        let right = sample * ((2.0 * PI * 0.5 * t).sin() * 0.15 + 0.85);
        (left, right)
    }

    /// Process a sample-accurate block of audio through all timeline tracks, summing into the mixer
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        tracks: &[Track],
        piano_roll_notes: &[FlPianoRollNote],
        mixer: &mut MixingEngine,
        start_frame: u64,
        block_len: usize,
        master_out_l: &mut [f32],
        master_out_r: &mut [f32],
    ) {
        let len = block_len.min(master_out_l.len()).min(master_out_r.len());
        if len == 0 {
            return;
        }

        // Ensure temporary track buffers are appropriately sized
        if self.track_buffer_l.len() < len {
            self.track_buffer_l.resize(len, 0.0);
            self.track_buffer_r.resize(len, 0.0);
        }

        // Reset bus accumulators for this audio slice
        mixer.clear_bus_accumulators(len);

        let end_frame = start_frame + len as u64;
        let start_tick = self.frame_to_tick(start_frame);
        let end_tick = self.frame_to_tick(end_frame);

        // Check whether any track is soloed
        let any_track_soloed = tracks.iter().any(|t| t.soloed);

        for (trk_idx, track) in tracks.iter().enumerate() {
            // Mute / Solo logic
            if track.muted || (any_track_soloed && !track.soloed) {
                continue;
            }

            // Zero out track work buffer
            self.track_buffer_l[..len].fill(0.0);
            self.track_buffer_r[..len].fill(0.0);

            let mut has_audio_on_track = false;

            // 1. Process Timeline Regions
            for region in &track.regions {
                if region.muted {
                    continue;
                }

                let reg_start_tick = region.start_tick;
                let reg_end_tick = reg_start_tick + region.length_ticks;

                // Check overlap
                if reg_end_tick <= start_tick || reg_start_tick >= end_tick {
                    continue;
                }

                let reg_start_frame = self.tick_to_frame(reg_start_tick);
                let reg_len_frames = self.tick_to_frame(region.length_ticks).max(1);
                let reg_end_frame = reg_start_frame + reg_len_frames;

                let fade_in_frames = self.tick_to_frame(region.fade.in_ticks);
                let fade_out_frames = self.tick_to_frame(region.fade.out_ticks);
                let region_linear_gain = if region.gain_db <= -96.0 {
                    0.0
                } else {
                    10.0f32.powf(region.gain_db / 20.0)
                };

                for i in 0..len {
                    let current_frame = start_frame + i as u64;
                    if current_frame >= reg_start_frame && current_frame < reg_end_frame {
                        let frame_in_reg = current_frame - reg_start_frame;

                        let fade_mult = Self::calculate_fade_multiplier(
                            frame_in_reg,
                            reg_len_frames,
                            fade_in_frames,
                            fade_out_frames,
                            region.fade.in_curve,
                            region.fade.out_curve,
                        );

                        let (raw_l, raw_r) = match &region.kind {
                            RegionKind::Audio { .. } => Self::synthesize_take_sample(
                                region.id,
                                trk_idx,
                                frame_in_reg,
                                self.sample_rate,
                            ),
                            RegionKind::Midi { .. } | RegionKind::Pattern { .. } => {
                                // Tone generation for MIDI/Pattern clip regions
                                let t = frame_in_reg as f32 / self.sample_rate;
                                let freq = 440.0 * 2.0f32.powf((60.0 - 69.0) / 12.0); // Middle C (261.63 Hz)
                                let s = (2.0 * PI * freq * t).sin() * 0.22;
                                (s, s)
                            }
                        };

                        let net_mult = fade_mult * region_linear_gain;
                        self.track_buffer_l[i] += raw_l * net_mult;
                        self.track_buffer_r[i] += raw_r * net_mult;
                        has_audio_on_track = true;
                    }
                }
            }

            // 2. Process Piano Roll Notes for Software Instrument / MIDI Tracks
            if track.track_type == TrackType::Software || track.track_type == TrackType::Midi {
                for note in piano_roll_notes {
                    if note.channel_id == trk_idx {
                        let note_start_frame = self.tick_to_frame(note.start_tick);
                        let note_len_frames = self.tick_to_frame(note.length_ticks).max(1);
                        let note_end_frame = note_start_frame + note_len_frames;

                        if note_end_frame <= start_frame || note_start_frame >= end_frame {
                            continue;
                        }

                        let freq = 440.0 * 2.0f32.powf((note.key as f32 - 69.0) / 12.0);
                        let vel = note.velocity.clamp(0.0, 1.0);

                        for i in 0..len {
                            let curr_frame = start_frame + i as u64;
                            if curr_frame >= note_start_frame && curr_frame < note_end_frame {
                                let frame_in_note = curr_frame - note_start_frame;
                                let t = frame_in_note as f32 / self.sample_rate;

                                // Note envelope: attack 10ms, decay
                                let env = (t / 0.01).min(1.0) * (-t * 1.5).exp().max(0.1);
                                let note_sample = (2.0 * PI * freq * t).sin() * 0.25 * vel * env;

                                let pan_l = if note.pan <= 0.0 { 1.0 } else { 1.0 - note.pan };
                                let pan_r = if note.pan >= 0.0 { 1.0 } else { 1.0 + note.pan };

                                self.track_buffer_l[i] += note_sample * pan_l;
                                self.track_buffer_r[i] += note_sample * pan_r;
                                has_audio_on_track = true;
                            }
                        }
                    }
                }
            }

            // 3. Route into Mixer Channel Strip and Calculate Ballistics
            if has_audio_on_track || trk_idx < mixer.channels.len() {
                mixer.route_channel_audio(
                    trk_idx,
                    &self.track_buffer_l[..len],
                    &self.track_buffer_r[..len],
                );
                if let Some(strip) = mixer.channels.get_mut(trk_idx) {
                    strip.meter.process_block(
                        &mut self.track_buffer_l[..len],
                        &mut self.track_buffer_r[..len],
                    );
                }
            }
        }

        // 4. Hierarchical bus summation into Master Stereo Out (processes MasterStrip)
        mixer.sum_buses(&mut master_out_l[..len], &mut master_out_r[..len]);
    }
}
