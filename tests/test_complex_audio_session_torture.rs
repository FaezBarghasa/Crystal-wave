use Crystal_wave::audio::playback_engine::TimelineAudioScheduler;
use Crystal_wave::bouncing::BounceEngine;
use Crystal_wave::fl_studio::FlPianoRollNote;
use Crystal_wave::mixing::MixingEngine;
use Crystal_wave::timeline::region::{FadeCurve, Region, RegionFade, RegionKind};
use Crystal_wave::timeline::track::{Track, TrackType};
use Crystal_wave::ui::message::UiMessage;
use Crystal_wave::ui::session_driver::UiSessionDriver;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::Cursor;

/// Helper to construct a production-grade 16-track complex project session
fn setup_16_track_production_session() -> (Vec<Track>, Vec<FlPianoRollNote>, MixingEngine) {
    let mut tracks = Vec::with_capacity(16);
    let mut notes = Vec::new();
    let mut mixer = MixingEngine::new(48000.0).expect("Mixer init");

    // Configure Aux Buses
    // Bus 1: Reverb Return
    mixer.buses[1].target_bus = Some(0);
    mixer.buses[1].volume_fader_db = -6.0;
    // Bus 2: Delay Return
    mixer.buses[2].target_bus = Some(0);
    mixer.buses[2].volume_fader_db = -9.0;
    // Bus 3: Drum Submix Summing Stack
    mixer.buses[3].is_summing_stack = true;
    mixer.buses[3].target_bus = Some(0);
    mixer.buses[3].volume_fader_db = 0.0;

    // Tracks 0..3: Drum Kit (routed to Drum Submix Bus 3)
    let drum_names = ["Kick Drum", "Snare Top", "Hi-Hat Closed", "Toms Floor"];
    for (i, name) in drum_names.iter().enumerate() {
        let mut trk = Track::new(i, *name, TrackType::Audio);
        trk.gain_db = -8.0 + (i as f32 * 0.5);
        trk.pan = if i == 1 {
            -0.2
        } else if i == 2 {
            0.3
        } else {
            0.0
        };

        // Add Audio region
        let mut reg = Region::new(
            i * 100 + 1,
            i,
            *name,
            0,
            1920 * 4,
            RegionKind::Audio {
                file_path: Some(format!("samples/drums/{}.wav", name.replace(' ', "_"))),
                sample_offset: 0,
                sample_length: 48000 * 4,
            },
        );
        reg.fade = RegionFade {
            in_ticks: 120,
            out_ticks: 240,
            in_curve: FadeCurve::EqualPower,
            out_curve: FadeCurve::SCurve,
        };
        trk.regions.push(reg);

        let ch_id = mixer.add_channel(*name).expect("Add channel");
        mixer.channels[ch_id].output_bus = 3;
        mixer.channels[ch_id].volume_fader_db = trk.gain_db;
        mixer.channels[ch_id].pan.balance = trk.pan;

        tracks.push(trk);
    }

    // Tracks 4..7: Organic Instruments (Bass, Acoustic Guitar, Piano, Strings)
    let inst_names = [
        "Electric Bass",
        "Acoustic Guitar",
        "Grand Piano",
        "Cello Section",
    ];
    for (idx, name) in inst_names.iter().enumerate() {
        let trk_id = idx + 4;
        let mut trk = Track::new(trk_id, *name, TrackType::Audio);
        trk.gain_db = -10.0;
        trk.pan = if idx % 2 == 0 { -0.4 } else { 0.4 };

        let reg = Region::new(
            trk_id * 100 + 1,
            trk_id,
            *name,
            480,
            1920 * 8,
            RegionKind::Audio {
                file_path: Some(format!("samples/inst/{}.wav", name.replace(' ', "_"))),
                sample_offset: 0,
                sample_length: 48000 * 8,
            },
        );
        trk.regions.push(reg);

        let ch_id = mixer.add_channel(*name).expect("Add channel");
        mixer.channels[ch_id].output_bus = 0; // Direct to Stereo Out
        mixer.channels[ch_id].volume_fader_db = trk.gain_db;
        mixer.channels[ch_id].pan.balance = trk.pan;

        tracks.push(trk);
    }

    // Tracks 8..11: Software Synthesizers (Polyphonic Chords, Arp, Lead, Pad)
    let synth_names = [
        "Poly Moog",
        "Jupiter Arp",
        "Acid 303 Lead",
        "Warm Brass Pad",
    ];
    for (idx, name) in synth_names.iter().enumerate() {
        let trk_id = idx + 8;
        let mut trk = Track::new(trk_id, *name, TrackType::Software);
        trk.gain_db = -12.0;

        let reg = Region::new(
            trk_id * 100 + 1,
            trk_id,
            *name,
            960,
            1920 * 8,
            RegionKind::Midi { clip_id: trk_id },
        );
        trk.regions.push(reg);

        // Add 4-note chord voicing for each synth
        for note_idx in 0..4 {
            notes.push(FlPianoRollNote {
                id: trk_id * 10 + note_idx,
                channel_id: trk_id,
                start_tick: 960 + (note_idx as u64 * 480),
                length_ticks: 960,
                key: 48 + (idx as u8 * 5) + (note_idx as u8 * 4),
                velocity: 0.8,
                pan: 0.0,
                color_group: 0,
                fine_pitch_cents: 0.0,
                is_slide: note_idx == 3, // 4th note is a slide articulation
            });
        }

        let ch_id = mixer.add_channel(*name).expect("Add channel");
        mixer.channels[ch_id].output_bus = 0;
        mixer.channels[ch_id].volume_fader_db = trk.gain_db;

        tracks.push(trk);
    }

    // Tracks 12..15: Vocals & FX (Lead Vox, Double Vox, Harmony Vox, FX Riser)
    let vocal_names = [
        "Lead Vocal",
        "Double Vocal",
        "Harmony Vox High",
        "FX Atmos Riser",
    ];
    for (idx, name) in vocal_names.iter().enumerate() {
        let trk_id = idx + 12;
        let mut trk = Track::new(trk_id, *name, TrackType::Audio);
        trk.gain_db = -8.0;

        let reg = Region::new(
            trk_id * 100 + 1,
            trk_id,
            *name,
            0,
            1920 * 16,
            RegionKind::Audio {
                file_path: Some(format!("samples/vox/{}.wav", name.replace(' ', "_"))),
                sample_offset: 0,
                sample_length: 48000 * 16,
            },
        );
        trk.regions.push(reg);

        let ch_id = mixer.add_channel(*name).expect("Add channel");
        // Route Vox to Stereo Out
        mixer.channels[ch_id].output_bus = 0;
        mixer.channels[ch_id].volume_fader_db = trk.gain_db;

        tracks.push(trk);
    }

    mixer.master_strip.volume_fader_db = -6.0;
    mixer.master_fader_db = -6.0;

    (tracks, notes, mixer)
}

#[test]
fn test_complex_16_track_session_playback_and_bus_summing() {
    let (tracks, notes, mut mixer) = setup_16_track_production_session();
    assert_eq!(tracks.len(), 16);
    assert_eq!(mixer.channels.len(), 16);

    let mut scheduler = TimelineAudioScheduler::new(48000.0, 128.0);
    let block_size = 256;
    let mut master_l = vec![0.0f32; block_size];
    let mut master_r = vec![0.0f32; block_size];

    // Process 20 consecutive blocks (simulate 5,120 audio frames = ~106 ms)
    for block_idx in 0..20 {
        let frame = (block_idx * block_size) as u64;
        scheduler.process_block(
            &tracks,
            &notes,
            &mut mixer,
            frame,
            block_size,
            &mut master_l,
            &mut master_r,
        );

        // Assert audio buffers are filled and finite
        for i in 0..block_size {
            assert!(master_l[i].is_finite());
            assert!(master_r[i].is_finite());
        }
    }

    // Verify MasterStrip meter ballistics are active and finite
    println!(
        "Master Meter: peak={:.2} dB, rms={:.2} dB, auto_mute={}",
        mixer.master_strip.meter.peak_db,
        mixer.master_strip.meter.rms_db,
        mixer.master_strip.meter.auto_mute_tripped
    );
    assert!(mixer.master_strip.meter.peak_db.is_finite());
    assert!(mixer.master_strip.meter.rms_db.is_finite());
    assert!(!mixer.master_strip.meter.auto_mute_tripped);
}

#[test]
fn test_master_strip_isolation_from_track_0() {
    let mut driver = UiSessionDriver::new(48000.0);

    // Initial check: Track 0 ("Audio 1") and Master Strip are independent
    let initial_track0_db = driver.app.mixer.channels[0].volume_fader_db;
    let initial_master_db = driver.app.mixer.master_strip.volume_fader_db;
    assert_eq!(initial_track0_db, 0.0);
    assert_eq!(initial_master_db, 0.0);

    // 1. Move Track 0 fader to -12.5 dB
    driver.dispatch(UiMessage::SetTrackFader {
        track_id: 0,
        volume_db: -12.5,
    });
    assert_eq!(driver.app.mixer.channels[0].volume_fader_db, -12.5);
    assert_eq!(
        driver.app.mixer.master_strip.volume_fader_db, 0.0,
        "Master strip fader must remain unaffected by Track 0 fader!"
    );

    // 2. Move Master fader (track_id = 9999) to -4.0 dB
    driver.dispatch(UiMessage::SetTrackFader {
        track_id: 9999,
        volume_db: -4.0,
    });
    assert_eq!(driver.app.mixer.master_strip.volume_fader_db, -4.0);
    assert_eq!(
        driver.app.mixer.channels[0].volume_fader_db, -12.5,
        "Track 0 fader must remain unaffected by Master strip fader!"
    );

    // 3. Mute Track 0: Master strip must remain unmuted
    driver.dispatch(UiMessage::ToggleTrackMute(0));
    assert!(driver.app.mixer.channels[0].muted);
    assert!(
        !driver.app.mixer.master_strip.muted,
        "Master strip must NOT be muted when Track 0 is muted!"
    );

    // 4. Reset auto mute on Track 0 does not mutate master
    driver.app.mixer.master_strip.meter.auto_mute_tripped = true;
    driver.dispatch(UiMessage::ResetAutoMute(0));
    assert!(
        driver.app.mixer.master_strip.meter.auto_mute_tripped,
        "Master strip auto mute must not reset when Track 0 auto mute resets!"
    );
    driver.dispatch(UiMessage::ResetAutoMute(9999));
    assert!(!driver.app.mixer.master_strip.meter.auto_mute_tripped);
}

#[test]
fn test_complex_session_torture_rapid_scrubbing_and_mutations() {
    let mut driver = UiSessionDriver::new(48000.0);
    driver.dispatch(UiMessage::TransportPlay);

    // Rapidly seek and mutate faders across 100 iterations
    for step in 0..100 {
        let seek_tick = ((step * 387) % 15360) as u64;
        driver.dispatch(UiMessage::SetPlayhead(seek_tick));

        // Adjust tempo dynamically
        let tempo = 60.0 + ((step % 18) as f32 * 10.0); // 60 BPM .. 230 BPM
        driver.dispatch(UiMessage::SetTempoBpm(tempo));

        // Random-like fader changes
        driver.dispatch(UiMessage::SetTrackFader {
            track_id: step % 2,
            volume_db: -30.0 + (step as f32 % 35.0),
        });

        // Tick audio synthesis
        driver.dispatch(UiMessage::Tick);

        assert!(driver.app.playhead_frame > 0);
        assert!(!driver.app.mixer.master_strip.meter.auto_mute_tripped);
    }
}

#[test]
fn test_offline_bounce_wav_16_track_session() {
    let (tracks, notes, mut mixer) = setup_16_track_production_session();
    let sample_rate = 48000.0;
    let mut scheduler = TimelineAudioScheduler::new(sample_rate, 128.0);
    let total_frames = 48000 * 2; // 2 seconds
    let block_size = 256;

    let mut full_left = Vec::with_capacity(total_frames);
    let mut full_right = Vec::with_capacity(total_frames);
    let mut block_l = vec![0.0f32; block_size];
    let mut block_r = vec![0.0f32; block_size];

    let mut frame = 0u64;
    while frame < total_frames as u64 {
        let len = block_size.min((total_frames as u64 - frame) as usize);
        scheduler.process_block(
            &tracks,
            &notes,
            &mut mixer,
            frame,
            len,
            &mut block_l,
            &mut block_r,
        );
        full_left.extend_from_slice(&block_l[..len]);
        full_right.extend_from_slice(&block_r[..len]);
        frame += len as u64;
    }

    // Apply normalization via BounceEngine
    let bounce = BounceEngine::default();
    bounce.apply_normalization(&mut full_left, &mut full_right);

    // Encode to 24-bit WAV using Hound
    let spec = WavSpec {
        channels: 2,
        sample_rate: sample_rate as u32,
        bits_per_sample: 24,
        sample_format: SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut buffer, spec).expect("Create WavWriter");
        let max_val = 8_388_607.0f32; // 24-bit max signed int
        for i in 0..full_left.len() {
            let sl = (full_left[i].clamp(-1.0, 1.0) * max_val) as i32;
            let sr = (full_right[i].clamp(-1.0, 1.0) * max_val) as i32;
            writer.write_sample(sl).expect("Write left");
            writer.write_sample(sr).expect("Write right");
        }
        writer.finalize().expect("Finalize WavWriter");
    }

    let bytes = buffer.into_inner();
    assert!(bytes.len() > 44);
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
}
