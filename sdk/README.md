# Crystal-Wave Universal Plugin SDK (Linux / Cross-Platform)

Develop native audio plugins, synths, and MIDI processors for **Crystal-Wave DAW** with the **Omid (Object-MIDI)** specification.

Supported languages:
- **Rust** (`export_crystal_plugin!` macro + `CrystalPlugin` trait)
- **C / C++** (`include/crystal_wave_plugin.h`)
- **Python / Ctypes** (`crystal_wave_plugin.py`)
- **Zig / Go / D / Swift / C#** (via standard C-ABI)

---

## 1. Rust Plugin Development

```rust
use Crystal_wave::plugins::rust_sdk::CrystalPlugin;
use Crystal_wave::export_crystal_plugin;
use omid::OmidPacket;

#[derive(Default)]
pub struct MyGainDistortion {
    gain: f32,
}

impl CrystalPlugin for MyGainDistortion {
    fn name(&self) -> &'static str { "Crystal Drive" }
    fn vendor(&self) -> &'static str { "MyStudio" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> u32 { 6 } // Distortion
    fn unique_id(&self) -> u32 { 9001 }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = (*l * self.gain).tanh();
            *r = (*r * self.gain).tanh();
        }
    }

    fn parameter_count(&self) -> usize { 1 }
    fn parameter_name(&self, _index: usize) -> &'static str { "Drive" }
    fn get_parameter(&self, _index: usize) -> f32 { self.gain }
    fn set_parameter(&mut self, _index: usize, val: f32) { self.gain = val; }

    fn on_omid_packet(&mut self, packet: OmidPacket) {
        // High-resolution 12-bit ADC fader or MIDI 2.0 control
        if packet.is_raw_data() {
            self.gain = packet.payload_as_normalized_f32(12) * 5.0;
        }
    }
}

export_crystal_plugin!(MyGainDistortion);
```

Build as a shared library:
```bash
cargo build --release --lib # produces libmy_plugin.so
```

---

## 2. C / C++ Plugin Development

Include `crystal_wave_plugin.h` and export `crystal_plugin_get_vtable`:

```c
#include "crystal_wave_plugin.h"
#include <math.h>
#include <stdlib.h>

typedef struct {
    float gain;
} MyGainState;

void my_process(void* instance, CrystalAudioBufferC* buffer) {
    MyGainState* state = (MyGainState*)instance;
    for (uint32_t i = 0; i < buffer->num_samples; ++i) {
        buffer->left_channel[i] = tanhf(buffer->left_channel[i] * state->gain);
        buffer->right_channel[i] = tanhf(buffer->right_channel[i] * state->gain);
    }
}

// Compile into .so
// gcc -O3 -shared -fPIC my_plugin.c -o my_plugin.so
```

---

## 3. Omid (Object-MIDI) Integration

Every plugin receives real-time 8-byte `OmidPacket` instances clock-locked to DSP buffers:
- **12-bit / 16-bit High-Resolution ADC values** (faders, knobs, pressure)
- **Bi-directional Haptic Force Feedback** (dynamic electromagnetic profiles)
- **Sub-millisecond latency** with zero memory allocation.
