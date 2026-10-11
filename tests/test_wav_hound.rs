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
