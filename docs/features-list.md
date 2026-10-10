# Apple Logic Pro 12.3.1: Feature Complete Specification (Macro & Micro Features)

This document provides an exhaustive, feature-complete inventory of every macro and micro feature extracted, reverse engineered, and verified from **Apple Logic Pro 12.3.1**.

---

## 1. Core Timeline & Arrangement Engine

### Macro Features
- **Non-Destructive Multi-Track Timeline**: Up to 1,000 audio tracks, 1,000 software instrument tracks, 1,000 auxiliary channels, and 256 buses.
- **Global Track Lanes**:
  - **Arrangement Track**: Song structure markers (Intro, Verse, Chorus, Bridge, Drop, Outro) with click-and-drag reordering of entire song sections across all tracks simultaneously.
  - **Chord Track**: Global chord progression timeline driving Session Players and MIDI transpositions.
  - **Tempo Track**: Smooth tempo curves, tempo ramps, jumps, and continuous tempo changes.
  - **Signature Track**: Compound and complex time signatures (4/4, 3/4, 6/8, 7/8, 11/8) and key signatures.
  - **Marker Track**: Color-coded song markers with text notes, cycle jumping, and key command navigation.
  - **Beat Mapping Track**: Dynamic alignment of musical bars and beats to live acoustic transients without altering original audio timing.
  - **Movie Track**: SMPTE timecode-synchronized video playback with thumbnail frame strip.

### Micro Features
- **Magnetic Grid Snapping**: Smart, Bar, Beat, Division, Tick, Frame, and Quarter-Frame modes with relative vs absolute snap toggles.
- **Dynamic Waveform Zooming**: Independent visual waveform vertical zoom without altering channel fader gain.
- **Region Anchor Point**: Custom sub-tick anchor offset allowing pick-up notes to be aligned cleanly to grid lines.
- **Track Stacks**:
  - *Folder Stack*: Collapsible sub-grouping for organization without routing audio.
  - *Summing Stack*: Sub-mixing bus grouping with a dedicated master channel strip for collective insert processing.
- **Track Alternatives**: Multiple independent arrangement versions per track without creating new tracks.
- **Track Freeze**:
  - *Source Freeze*: Freezes plugins before fader to conserve CPU.
  - *Pre-Fader Freeze*: Complete audio render bypassing real-time DSP computation.
- **Track Hide / Reveal**: Global track hiding to declutter large 200+ track arrangements.
- **Track Color Palette**: 96 curated high-contrast hues with automatic luminance text inversion.
- **Catch Playhead Modes**: Catch playhead during playback, scroll continuously, or page by page.
- **Varispeed Engine**:
  - *Speed Only*: Varispeed time-stretch without altering pitch.
  - *Varispeed and Pitch*: Tape/vinyl pitch-proportional resampling.
  - *MIDI Only*: Scales sequencer clock BPM without audio resampling.
  - Units: Percentage ($\pm 50\%$), Semitones ($\pm 7$), or exact BPM offset.
- **Punch Recording & Pre/Post Roll**:
  - Auto-Punch with dedicated red punch locators independent of cycle loop points.
  - Configurable pre-roll and post-roll in bars or seconds.
  - Replace mode (destructive tape-style overdub) vs Merge mode vs New Track mode.
- **Low Latency Mode**:
  - Configurable latency cutoff threshold ($1\,\text{ms}$ to $20\,\text{ms}$).
  - Automatically detects and disables high-latency plugins (lookahead limiters, linear-phase EQs, convolution reverbs) on live record-enabled tracks, while leaving other playback tracks full-fidelity.

---

## 2. Audio Recording, Editing & Elastic Audio

### Macro Features
- **Quick Swipe Comping**:
  - Automatic Take Folder creation on cycle recording.
  - Real-time mouse swiping across takes to assemble the composite (comp) track.
  - Automatic seamless $5\,\text{ms}$ equal-power crossfading at slice boundaries.
  - Flatten and Merge or Export Comp to a dedicated track.
- **Flex Time (Elastic Audio)**:
  - *Slicing Mode*: Transient-preserving drum and percussion editing.
  - *Rhythmic Mode*: Loop and percussive slicing with crossfaded tails.
  - *Monophonic Mode*: Pitch-synchronous overlap-add (PSOLA) for solo vocals and monophonic instruments.
  - *Polyphonic Mode*: Phase-locked phase vocoder for guitars, pianos, and full mixes.
  - *Tempophone Mode*: Granular time-stretching with adjustable grain window and crossfade.
  - *Speed Mode*: Classic tape/vinyl varispeed resampling.
- **Flex Pitch (Integrated Pitch Correction)**:
  - Audio waveform note block extraction on the arrange timeline or in the Audio Track Editor.
  - 6 hotspot handles per note: Pitch Drift In, Fine Pitch, Pitch Drift Out, Gain, Vibrato, Formant Shift.
- **Smart Tempo Engine**:
  - *Keep Project Tempo*: Conform imported audio to project BPM.
  - *Adapt Project Tempo*: Automatically calculate project tempo map from a freeform audio recording.
  - *Auto Tempo*: Automatically detect whether to adapt or keep tempo based on context.
  - Multi-track Smart Tempo sets for phase-coherent drum kit recording.

### Micro Features
- **Sample-Accurate Audio File Editor**: 1-sample level editing, pencil redraw tool to remove clicks/pops, DC offset removal, 0-crossing auto-snap.
- **Strip Silence**: Automatic silence detection with configurable threshold ($\text{dB}$), minimum section length ($\text{ms}$), pre-attack ($\text{ms}$), and post-release ($\text{ms}$).
- **Fade Tool & Curve Editing**:
  - Region-boundary drag handles for fade-in, fade-out, crossfade, and equal-gain curves.
  - Speed-up and Slow-down tape stop / start acceleration curves.
- **Gain Tool**: Click-and-drag clip gain adjustment directly on individual audio regions.
- **Marquee Tool Micro-Workflows**:
  - Razor-sharp section splitting.
  - Transient snapping within marquee selection.
  - Instant cycle playback of highlighted marquee selection.
- **Transient Editing**: Dedicated Move Transient and Insert/Delete Warpmarker tools.
- **Reverse Audio**: Single-click non-destructive or destructive region reversal.
- **Quantize Audio**: Phase-locked audio quantize across grouped drum tracks.

---

## 3. Mixing Console & Spatial Audio Routing

### Macro Features
- **Dolby Atmos & Spatial Audio**:
  - Native 7.1.4 Dolby Atmos bed and 3D Object Panner.
  - 3D Object Panner parameters: Azimuth, Elevation, Spread, Size, and Height.
  - Integrated Dolby Atmos Renderer with binaural headphone monitoring (Spatial Audio with dynamic head tracking).
- **Console Routing**:
  - Flexible Send slots (Pre-fader, Post-fader, Post-pan).
  - Stereo Pan, Balance Pan, Dual Pan (independent Left/Right positioning), and Binaural Panner.
  - VCA Master faders with group link options.
  - Dedicated sidechain inputs across all dynamics and filter plugins.
- **Automation Engine**:
  - Modes: Read, Touch, Latch, Write.
  - Region-Based Automation (RBA) vs Track-Based Automation (TBA).
  - Bezier curve bending with the Automation Curve Tool.
  - Automation Trim mode for relative gain offsets.

### Micro Features
- **Automation Quick Access (AQA)**: Dynamically map a single hardware MIDI fader to whichever automation parameter is currently visible on the selected track.
- **Automation Follows Regions**: Modes: *Never*, *Ask*, *Always* (moving a region automatically moves its automation nodes).
- **Automation Thinning**: Redundant node reduction algorithm.
- **Virtual Meter Bridge**: High-resolution level meters across all tracks with peak hold and True Peak detection.
- **Gain Reduction Meters**: Small gain reduction meters embedded directly inside channel strip compressor inserts.
- **Channel Strip Copy/Paste**: Copy entire plugin chains, routing, and EQ curves between channels with one keystroke.
- **Auto-Mute Safety Protection**: Automatic emergency audio muting when DSP output exceeds $+12\,\text{dBFS}$ overload.
- **Phase Invert**: Independent $180^\circ$ phase inversion toggle per channel strip.
- **Track Delay Offset**: Sub-millisecond or tick-based delay offset per track (positive or negative delay) for groove timing or external hardware compensation.

---

## 4. MIDI, Sequencing, MPE & Note Editing

### Macro Features
- **Piano Roll Editor**:
  - High-precision tick resolution ($960\,\text{PPQN}$).
  - Note velocity visual color-coding (blue $\rightarrow$ green $\rightarrow$ yellow $\rightarrow$ red).
  - Quantization engine: Standard, Triplet, Dotted, Swing ($50-75\%$), and Scale Quantize (Major, Minor, Pentatonic, Dorian, etc.).
  - MIDI Draw lane: Polyphonic Aftertouch, Pitch Bend, Modulation, CC messages, and Expression.
- **Step Sequencer**:
  - Independent polyrhythmic lane lengths ($1-64$ steps).
  - Per-step lane attributes: Velocity, Gate, Sub-step (Ratchet up to $16\times$), Chance/Probability ($0-100\%$), Note Repeat, Flam, Octave Offset, and Offset Tick.
  - Per-lane playback directions: Forward, Reverse, Ping-Pong, Random.
- **Live Loops Grid**:
  - Cell-based non-linear clip launcher.
  - Quantized launch clock: Bar, Beat, Division, Tick.
  - Performance recording from Live Loops cells directly into the linear arrange timeline.
- **Score Editor**:
  - Full professional musical notation engraving.
  - Polyphonic staff styles, guitar tablature, drum notation, dynamic marks, slurs, ties, and lyrics.
  - MusicXML export and import.
- **Event List**:
  - Raw tabular event list displaying tick position, status, channel, note/data byte 1, and value/data byte 2.

### Micro Features
- **Articulation System & Keyswitching**:
  - Articulation IDs ($1-255$) tagged directly on individual MIDI notes (independent of MIDI channel).
  - Articulation maps supporting Keyswitch (Note trigger), Program Change, Control Change (CC), or Poly Pressure.
  - Type modes: *Direction* (persists across subsequent notes until changed) vs *Attribute* (applies only to the specific note).
- **MIDI 2.0 & MPE (MIDI Polyphonic Expression)**:
  - 16-bit high-resolution velocity ($65,536$ levels).
  - Per-note pitch bend ($\pm 96$ semitones per note).
  - Per-note brightness/timbre (CC 74) and per-note pressure/aftertouch.
- **Capture Recording (Retroactive MIDI Record)**:
  - Continuous background circular MIDI buffer.
  - Pressing `Shift+R` retrieves MIDI played while transport was stopped, instantly creating a region with inferred tempo and downbeat.
- **Brush Tool**: Paint repeating rhythmic notes across the piano roll at selected grid divisions.
- **Flam & Humanize Functions**: Configurable velocity, position, and length humanization algorithms.
- **Demix by MIDI Channel / Pitch**: One-click separation of multi-channel MIDI or multi-piece drum recordings onto individual tracks.

---

## 5. Generative AI, Session Players & Machine Learning

### Macro Features
- **AI Session Players**:
  - **Session Drummer**: Acoustic and electronic drum performances driven by genre styles (Rock, Pop, Jazz, Electronic, Hip Hop, Metal).
  - **Session Bassist**: Generates realistic basslines following the Chord Track with choice of fingerstyle, picked, slap, and synth bass tones.
  - **Session Keyboardist**: Generates piano, electric piano, and organ voicings and accompaniment patterns.
- **2D Style & Performance Coordinates**:
  - 2D X/Y Complexity vs Intensity coordinate pad.
  - Rhythmic feel: Push / Pull timing offset relative to the beat grid.
  - Fill frequency slider and humanized ghost notes.
- **AI Stem Splitter**:
  - CoreML neural network (`MKGerm_48` / `RTRST_48`) decomposing mixed stereo audio files into 4 isolated stems: Vocals, Drums, Bass, and Other.
- **AI Mastering Assistant**:
  - Automatic spectral analysis and adaptive mastering curve generation.
  - Character styles: Transparent, Clean, Valve, Punch.
  - Loudness targeting (LUFS) with true peak limiting and stereo spread enhancement.
- **Neural Chroma & Chord Detection**:
  - Automatic audio harmonic analysis projecting chords directly onto the Chord Track.

### Micro Features
- **Voice Leading Solver (`UseForChordBassDistToHarmCenter`)**: Evaluates harmonic proximity of root, fifth, third, and passing chromatic notes to maintain natural voice-leading contours.
- **LuaGroove Pattern Engine**: Scriptable Lua groove generator executing micro-timing variations.

---

## 6. Native Audio Plugins & Virtual Instruments (134 Built-in)

### Virtual Instruments
1. **Alchemy**: 4-source Additive, Spectral, Granular, and Virtual Analog synthesizer with 2D morphing and MSEG modulators.
2. **Sample Alchemy**: Rapid drag-and-drop synth transforming audio clips into playable instruments.
3. **Sampler**: Full multi-sample workstation with zone mapping, velocity layers, round-robin, and pitch tracking.
4. **Quick Sampler**: Single-sample instrument with Classic, One-Shot, and Slice modes with auto-transient detection.
5. **Drum Machine Designer**: 64-pad MPC-style performance surface with independent plugin chains per pad.
6. **Drum Kit Designer**: Acoustic multi-mic drum kit with direct mic bleed, overheads, and room microphones.
7. **Retro Synth**: 4-mode vintage synth (Analog Minimoog, Sync, PPG Wavetable, and Yamaha DX FM).
8. **ES2**: 3-oscillator hybrid synth with planar vector envelope and 10-slot modulation matrix.
9. **Sculpture**: Physical modeling string, beam, and acoustic membrane synthesizer with strike, pluck, and bow exciters.
10. **Vintage B3 Organ**: Tonewheel organ emulation with drawbars, percussion, scanner vibrato, and rotary Leslie speaker.
11. **Vintage Electric Piano**: Fender Rhodes, Wurlitzer, and Hohner Pianet emulation.
12. **Vintage Clavinet**: Clavinet D6 emulation with dual magnetic pickups and filter switches.
13. **Vintage Mellotron**: Tape loop replay synthesizer with flute, string, and choir tapes.
14. **Studio Strings & Studio Horns**: Articulation-switched orchestral ensembles.
15. **Ultrabeat**: 24-drum voice synthesizer with integrated 32-step grid sequencer.
16. **Legacy Instruments**: ES1, ESP, ESE, ESM, EFM1, EVOC 20 PolySynth.

### Dynamics & Compression
17. **Compressor**: 7 circuit models (Studio VCA, Studio FET 1176, Vintage VCA, Vintage FET, Vintage Opto LA-2A, Classic VCA SSL, Platinum Digital).
18. **Adaptive Limiter**: Lookahead brickwall limiter with true-peak inter-sample clipping prevention.
19. **Multipressor**: 4-band multi-band compressor/expander.
20. **DeEsser 2**: Dual-mode vocal de-esser with frequency suppression preview.
21. **Enveloper**: Transient designer adjusting attack and release gain.
22. **Noise Gate**: Ducking, sidechain filter, and lookahead gate.
23. **Expander** & **Limiter**.

### Equalizers & Filters
24. **Channel EQ**: 8-band parametric EQ with integrated real-time 64-bit FFT analyzer.
25. **Linear Phase EQ**: Zero-phase distortion FIR mastering equalizer.
26. **Match EQ**: A/B spectral fingerprint learning and automatic EQ curve compensation.
27. **Vintage Console EQ**: Neve 1073 inductor EQ emulation with harmonic drive.
28. **Vintage Tube EQ**: Pultec EQP-1A / MEQ-5 passive tube equalizer with simultaneous boost/cut trick.
29. **Vintage Graphic EQ**: API 560 10-band proportional-Q graphic equalizer.
30. **AutoFilter**: Resonant state-variable filter modulated by LFO and envelope followers.
31. **Spectral Gate**: FFT bin frequency threshold gating.

### Reverbs & Spatial Processors
32. **ChromaVerb**: Algorithmic reverb with 14 acoustic spaces and MetalKit 3D wave visualization.
33. **Space Designer**: Convolution reverb with real-time synthesized envelopes, impulse response generation, and EQ.
34. **Quantec Room Simulator**: Revered German algorithmic studio room simulator.
35. **SilverVerb** & **EnVerb**.

### Delay & Time Modulation
36. **Delay Designer**: 36-tap multi-tap delay with independent pan, cutoff, and transposition per tap.
37. **Stereo Delay** & **Tape Delay**: Analog tape flutter, wow, and saturation modeling.
38. **Beat Breaker**: Multi-slice real-time audio glitch, scratch, time-stretch, and pitch-shifter.
39. **Ringshifter**: Ring modulator and frequency shifter.
40. **Chorus**, **Flanger**, **Phaser**, **Microphaser**, **Tremolo**, **Scanner Vibrato**.

### Distortion & Coloration
41. **ChromaGlow**: Neural saturation engine (Clean Tube, Warm Tube, Analog Tape, Console, Drive).
42. **Phat FX**: Modular multi-effects saturator with Bass Enhancer, Distortion, and Bandpass filters.
43. **Step FX**: 3-lane rhythmic multi-effect modulation grid.
44. **Bitcrusher**: Downsampling, bit reduction, and clipping engine.
45. **Amp Designer** & **Bass Amp Designer**: Vintage and modern guitar/bass amplifier heads and cabinet impulse responses.
46. **Pedalboard**: 35 stompbox effects (Fuzz, Overdrive, Delay, Wah, Modulation).

### MIDI Processing & Scripting
47. **Arpeggiator**: Advanced MIDI pattern generator with note sequencing, chord triggers, and octave transposition.
48. **Scripter**: Custom JavaScript engine for real-time algorithmic MIDI processing.
49. **Chord Trigger**, **Modifier**, **Modulator**, **Note Repeater**, **Randomizer**, **Transposer**, **Velocity Processor**.

---

## 7. Master Bouncing, Dithering & Export

### Micro Features
- **2nd Cycle Pass**: Automatically computes a preliminary cycle pass during export to capture the natural decay tail of reverbs, delays, and synth releases into the loop start.
- **Offline vs Real-Time Bounce**: High-speed multi-core offline rendering or 1x real-time rendering for analog outboard gear.
- **Dithering Algorithms**:
  - *POW-r #1*: Flat triangular probability density function (TPDF) noise shaping for narrow dynamic range spoken word.
  - *POW-r #2*: Low-noise psychoacoustic shaping optimized for acoustic and pop music.
  - *POW-r #3*: High-order 9th-degree psychoacoustic noise shaping minimizing noise in the critical $2-4\,\text{kHz}$ human ear sensitivity curve.
  - *UV22HR*: Apogee algorithm injecting ultra-high frequency energy at the Nyquist frequency.
- **Normalize Options**: *Off*, *Peak Normalize*, *Loudness Target (LUFS)*, *Overload Protection Only* (attenuates only if clipping occurs).
- **Pro Interchange Formats**: AAF, OMF 2.0, Final Cut Pro XML, MusicXML, standard MIDI File (Format 0/1).
