use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::io::Cursor;

#[test]
fn test_hound_wav_encode_and_decode() {
    let spec = WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut buffer, spec).expect("Create WavWriter");
        // Write 100 stereo samples of sine wave
        for i in 0..100 {
            let sample_l = ((i as f32 * 0.1).sin() * 30000.0) as i16;
            let sample_r = ((i as f32 * 0.1).cos() * 30000.0) as i16;
            writer.write_sample(sample_l).unwrap();
            writer.write_sample(sample_r).unwrap();
        }
        writer.finalize().expect("Finalize WavWriter");
    }

    let bytes = buffer.into_inner();
    assert!(bytes.len() > 44); // Standard RIFF header is at least 44 bytes
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");

    // Read back using WavReader
    let reader_cursor = Cursor::new(bytes);
    let mut reader = WavReader::new(reader_cursor).expect("Create WavReader");
    assert_eq!(reader.spec().channels, 2);
    assert_eq!(reader.spec().sample_rate, 44100);
    assert_eq!(reader.spec().bits_per_sample, 16);

    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    assert_eq!(samples.len(), 200); // 100 frames * 2 channels
}

#[test]
fn test_exporter_hound_wav_roundtrip() {
    use Crystal_wave::audio::AudioDecoder;
    use Crystal_wave::bouncing::{BitDepth, DitherType, ExportFormatConfig, Exporter};

    let cfg = ExportFormatConfig {
        sample_rate: 48000.0,
        ..Default::default()
    };
    let exporter = Exporter::new(cfg, BitDepth::Bit16, DitherType::None);

    let left = vec![0.5f32; 256];
    let right = vec![-0.5f32; 256];

    let wav_bytes = exporter
        .export_stereo_wav(&left, &right, 48000.0)
        .expect("Export WAV failed");

    assert!(wav_bytes.len() > 44);
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    assert_eq!(&wav_bytes[8..12], b"WAVE");

    let decoded = AudioDecoder::decode_wav_bytes(&wav_bytes).expect("Decode WAV failed");
    assert_eq!(decoded.channels, 2);
    assert_eq!(decoded.sample_rate, 48000);
    assert_eq!(decoded.samples_interleaved.len(), 512);

    let (dec_l, dec_r) = decoded.to_planar_stereo();
    assert_eq!(dec_l.len(), 256);
    assert_eq!(dec_r.len(), 256);
    // Amplitude should be approximately 0.5 and -0.5
    assert!((dec_l[0] - 0.5).abs() < 0.05);
    assert!((dec_r[0] - (-0.5)).abs() < 0.05);
}
