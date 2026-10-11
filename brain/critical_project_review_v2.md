# Brutal & Pessimistic Engineering Audit: Crystal-Wave

**Audit Scope:** Every module, crate, feature, dependency, menu, and implementation detail.  
**Engineering Posture:** Hyper-critical, systems-first, zero-sugarcoating.  
**Auditor Roles:** Senior Systems Architect, Audio DSP Engineer, Ponytail Bloat Auditor.

---

## 1. The Core Diagnosis: "Architectural Potemkin Village"

Crystal-Wave compiles cleanly and boasts 59 passing unit tests. However, under strict systems-engineering scrutiny, the project is largely an **Architectural Potemkin Village**: a collection of well-typed Rust structs and isolated domain models that **do not form a functioning, deployable DAW**.

1. **The DAW Cannot Output Audio to Hardware:**
   - `cpal` is declared in `Cargo.toml` with `features = ["asio"]`.
   - **Zero lines of code in `src/` call `cpal`.**
   - There is no audio stream callback, no ALSA/JACK/PipeWire/CoreAudio/WASAPI binding, and no hardware device enumerator. The engine is entirely offline and headless.
2. **Ghost Dependencies & Ponytail Bloat:**
   - Massive crates like `wgpu = "30.0"`, `clack-host = "0.2"`, `rkyv = "0.8"`, and `basedrop = "0.1"` are compiled in `Cargo.toml`, inflating build times and binary size by tens of megabytes.
   - **None of them are used anywhere in `src/`.**
3. **Island Modules (Disconnected Architecture):**
   - `pro_tools` (Smart Tool, VCA groups, grouped playlists, EBU R128 meter, I/O matrix) is implemented in isolation.
   - `fl_studio` (Channel Rack, Piano Roll, Gross Beat, Patcher, Soft Clipper) is implemented in isolation.
   - **Neither subsystem is wired into `CrystalWaveApp` (`src/ui/app.rs`) or `main.rs`.** A user launching the app cannot interact with any of these features.
4. **`main.rs` is a Dead Print Script:**
   - `src/main.rs` only prints 6 lines of text to stdout and exits. It does not launch the Iced GUI event loop (`CrystalWaveApp::run()`) or start an audio server.

---

## 2. Module-by-Module Autopsy

### 2.1 Audio & Real-Time Path (`src/audio/`, `src/mixing/`)
* **Real-Time Safety in Theory vs. Reality:**
  - `test_realtime_safety.rs` proves that `rtrb::RingBuffer` works. But in `MixingEngine::process_mix_buffer`, channels and tracks are managed via standard collections. If channel additions occur while playing, reallocations occur.
  - No hardware DAC clock synchronization: Audio processing is only driven when a test calls `.process()`. There is no monotonic timer, no jitter buffer, and no lock-free audio thread supervisor.
* **Master Bus Summing & Incomplete Pan Law:**
  - `DolbyAtmosPanner` calculates 7.1.4 bed gains via basic spherical trigonometry, but there is no HRTF (Head-Related Transfer Function) binaural renderer for headphones. Without HRTF, 7.1.4 monitoring on standard stereo headphones sounds identical to flat stereo panning.

### 2.2 DSP & Algorithms (`src/dsp/`, `src/elastic/`, `src/bouncing/`)
* **Elastic Time-Stretching (`src/elastic/time_stretch.rs`):**
  - PSOLA and granular algorithms allocate vectors (`vec![0.0f32; hanning_len]`, `vec![0.0f32; grain_len]`) on every single function invocation.
  - Lacks SIMD vectorization: Hann window evaluation calculates trigonometric `.cos()` sequentially inside a single-threaded CPU loop.
  - Transient smearing: The slicing mode uses arbitrary hardcoded 512-sample attack windows without energy-based onset peak tracking.
* **Compressor & Equalizer (`src/dsp/compressor.rs`, `equalizer.rs`):**
  - Compressor envelope detector uses linear decibel conversions per-sample with multiple `powf` and `log10` calls in the tight inner loop, which stalls the CPU FPU pipeline.
  - Equalizer biquads lack coefficient smoothing: Sweeping an EQ band with an automation curve causes audible digital clicks (zipper noise) because coefficients step instantaneously between blocks.
* **EBU R128 Loudness (`src/dsp/loudness_ebu_r128.rs`):**
  - Correct filter coefficients and gating mathematics, but accumulates unbounded memory in `block_energies: Vec<f32>` over long playback sessions. Running the DAW for an 8-hour mixing session will leak hundreds of megabytes in un-compacted block energies.

### 2.3 Plugin Architecture & FFI (`src/plugins/`)
* **Mock Host Only:**
  - `plugin_host.rs` defines a `PluginInstance` trait and a built-in `DelayPlugin`.
  - There is no dynamic `.so` / `.vst3` / `.clap` loader implemented using `libloading` or `clack-host`.
  - External plugins cannot be scanned from disk, loaded into memory, or visually rendered.

### 2.4 User Interface & Event Loop (`src/ui/`, `src/ui_state/`)
* **Disconnected Iced 0.14 View:**
  - The UI provides tabs and drawers for "Piano Roll", "Step Sequencer", and "Mixer", but they render hardcoded static widgets.
  - The newly implemented Pro Tools Smart Tool and FL Studio Channel Rack/Patcher have **zero visual widget representations** in `src/ui/view.rs`.
  - No keyboard focus hook: Pressing 'B', 'A', 'S' on the keyboard in the UI does not trigger `handle_keyboard_focus_key` because `UiMessage` lacks single-key focus routing.

### 2.5 Generative / AI Modules (`src/generative/`)
* **The "AI" Moniker is Misleading:**
  - `ai_melody.rs`, `ai_drum_kit.rs`, `ai_bass.rs`, `ai_chords.rs`, `ai_lyrics.rs` are standard random number generators (`rand::thread_rng()`) and primitive static lookup tables.
  - There is no ONNX runtime, no quantized transformer model, no neural network, and no machine learning pipeline. Calling these "AI" is misleading engineering nomenclature.

---

## 3. Strict Ponytail Audit (Over-Engineering & Dead Code)

| Component / Crate | Diagnostic | Action Required |
| :--- | :--- | :--- |
| `wgpu = "30.0"` in `Cargo.toml` | **Unused Ghost Dependency** | Remove from `Cargo.toml` or implement actual GPU timeline renderer. |
| `clack-host = "0.2"` in `Cargo.toml` | **Unused Ghost Dependency** | Remove or implement real CLAP hosting. |
| `rkyv = "0.8"` in `Cargo.toml` | **Unused Ghost Dependency** | Remove or replace JSON/bincode project serialization with rkyv zero-copy. |
| `basedrop = "0.1"` in `Cargo.toml` | **Unused Ghost Dependency** | Remove or wire into lock-free audio thread deallocation. |
| `src/generative/ai_*.rs` (14 files) | **Over-engineered heuristic stubs** | Consolidate into a single `src/midi/generators.rs` module. |
| `src/main.rs` | **Dead CLI Stub** | Rewrite to launch `CrystalWaveApp::run()` with audio device initialization. |

---

## 4. Prioritized Engineering Roadmap to True Viability

1. **Phase 1: Real Hardware Audio Loop**
   - Implement `AudioBackend` using `cpal` to open real ASIO/ALSA/WASAPI/CoreAudio output streams.
   - Run the audio callback in a high-priority real-time thread feeding `MixingEngine` blocks into the DAC.
2. **Phase 2: Purge Ghost Dependencies (Ponytail)**
   - Strip unused crates (`wgpu`, `clack-host`, `rkyv`, `basedrop`) or implement their minimal functional integration.
3. **Phase 3: Deep Wire UI with Domain Engines**
   - Connect `src/pro_tools` (Smart Tool, Memory Locations, VCA) and `src/fl_studio` (Channel Rack, Piano Roll) into `CrystalWaveApp` view and update reducers.
   - Wire single-keystroke keyboard focus directly into Iced keyboard subscription events.
4. **Phase 4: DSP Hardening**
   - Vectorize inner loops with SIMD.
   - Implement parameter smoothing (one-pole low-pass on gain and biquad filter frequencies).
   - Bound historical memory in `EbuR128LoudnessMeter`.
