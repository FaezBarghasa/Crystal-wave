pub mod ai_bass;
pub mod ai_chord_progression;
pub mod ai_chords;
pub mod ai_counter_melody;
pub mod ai_drum_kit;
pub mod ai_guitar;
pub mod ai_harmony;
pub mod ai_harmony_generation;
pub mod ai_keyboards;
pub mod ai_lyric_generation;
pub mod ai_lyrics;
pub mod ai_melody;
pub mod ai_melody_generation;
pub mod ai_piano;
pub mod ai_synth;
pub mod ai_vocals;
pub mod ai_voice_leading;
pub mod session_player;

pub use ai_bass::{AiBass, BassPlayStyle};
pub use ai_chord_progression::{HarmonicTensionSolver, MusicalCadence};
pub use ai_chords::AiChords;
pub use ai_counter_melody::AiCounterMelody;
pub use ai_drum_kit::{AiDrumKit, DrumGenre};
pub use ai_guitar::{AiGuitar, StrumDirection};
pub use ai_harmony::{AiHarmony, HarmonyInterval};
pub use ai_harmony_generation::{FourPartHarmonizer, SatbVoicing};
pub use ai_keyboards::{AiKeyboards, KeyboardInstrumentType};
pub use ai_lyric_generation::{AiLyricGeneration, LyricMeterPattern, PoeticMeter};
pub use ai_lyrics::{AiLyrics, LyricLine, RhymeScheme};
pub use ai_melody::{AiMelody, MelodicContour};
pub use ai_melody_generation::MotifGenerator;
pub use ai_piano::{AiPiano, PianoCompingStyle};
pub use ai_synth::{AiSynth, SynthPatternType};
pub use ai_vocals::{AiVocals, VocalRange};
pub use ai_voice_leading::VoiceLeadingSolver;
pub use session_player::{
    GeneratedMidiNote, GenerativePerformanceParams, SessionPlayer, SessionPlayerType,
};
