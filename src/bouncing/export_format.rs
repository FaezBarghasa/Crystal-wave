#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioExportFormat {
    #[default]
    Wav,
    Aiff,
    Flac,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProInterchangeFormat {
    AAF,
    FinalCutProXML,
    MusicXML,
    StandardMidiFormat0,
    StandardMidiFormat1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportFormatConfig {
    pub audio_format: AudioExportFormat,
    pub interchange: Option<ProInterchangeFormat>,
    pub sample_rate: f32,
    pub include_tempo_map: bool,
    pub include_markers: bool,
}

impl Default for ExportFormatConfig {
    fn default() -> Self {
        Self {
            audio_format: AudioExportFormat::Wav,
            interchange: None,
            sample_rate: 44100.0,
            include_tempo_map: true,
            include_markers: true,
        }
    }
}
