#![allow(non_snake_case)]

pub mod audio;
pub mod audio_edit;
pub mod bouncing;
pub mod dsp;
pub mod elastic;
pub mod error;
pub mod fl_studio;
pub mod format;
pub mod generative;
pub mod harmony;
pub mod instruments;
pub mod midi;
pub mod mixer;
pub mod mixing;
pub mod os;
pub mod plugins;
pub mod pro_tools;
pub mod smart_controls;
pub mod timeline;
pub mod ui;
pub mod ui_state;

// Re-exports for convenience and backwards-compatibility
pub use dsp::compressor::{Compressor, CompressorConfig, CompressorModel, MultibandCompressor};
pub use dsp::equalizer::{BiquadCoeffs, ChannelEq, EqBand, FilterType};
pub use dsp::flex::{FlexMode, FlexPitchNote, FlexProcessor};
pub use dsp::loudness_ebu_r128::EbuR128LoudnessMeter;
pub use dsp::reverb::ChromaVerbEngine;
pub use error::{LogicError, Result};
pub use fl_studio::{
    AutomationClip, ChannelRack, FlBrowser, FlHistoryManager, FlPatcher, FlPattern,
    FlPianoRollNote, FlPianoRollTools, FlPlaylist, FlWindowManager, FruitySoftClipper,
    GrossBeatEngine, GrossBeatTimePreset, GrossBeatVolumePreset, PlaylistItem, SplineNode,
};
pub use generative::{GenerativePerformanceParams, SessionPlayer, SessionPlayerType};
pub use harmony::{Chord, ChordQuality, ChordTrack, MarkovProgressionEngine};
pub use midi::{MidiDirection, SequencerStep, StepSequencerLane};
pub use mixer::{ChannelStrip, DolbyAtmosPanner, PanMode, PanPosition, SendMode, SendSlot};
pub use os::{ThreadPriorityManager, ThreadPriorityStatus};
pub use pro_tools::{
    ClipGeometry, GroupedTake, HardwareInsert, IoMatrix, KeyboardFocusCommand, MemoryLocation,
    MemoryLocationBank, MultiTrackPlaylistGroup, PhysicalPort, PortDirection, ProToolsClip,
    SmartToolAction, SmartToolContext, VcaManager, VcaMaster,
};
pub use smart_controls::{ParameterMapping, SmartControlsLayout, SmartKnob};
pub use timeline::{PPQN, TakeFolder, Timeline};
pub use ui::{CrystalWaveApp, UiMessage, UiSessionDriver};
pub use ui_state::{DualChannelInspector, EditorDrawerTab, LcdDisplayMode, WorkspaceState};
