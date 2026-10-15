#ifndef CRYSTAL_WAVE_PLUGIN_H
#define CRYSTAL_WAVE_PLUGIN_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

#define CRYSTAL_PLUGIN_API_VERSION 1

// Plugin Categories matching DAW architecture
typedef enum {
    CRYSTAL_CAT_DYNAMICS = 0,
    CRYSTAL_CAT_EQUALIZER = 1,
    CRYSTAL_CAT_FILTER = 2,
    CRYSTAL_CAT_REVERB = 3,
    CRYSTAL_CAT_DELAY = 4,
    CRYSTAL_CAT_MODULATION = 5,
    CRYSTAL_CAT_DISTORTION = 6,
    CRYSTAL_CAT_PITCH_CORRECTION = 7,
    CRYSTAL_CAT_UTILITY = 8,
    CRYSTAL_CAT_SYNTH = 9,
    CRYSTAL_CAT_SAMPLER = 10,
    CRYSTAL_CAT_MIDI_EFFECT = 11,
} CrystalPluginCategory;

// C-compatible Plugin Metadata
typedef struct {
    uint32_t api_version;
    uint32_t unique_id;
    const char* name;
    const char* vendor;
    const char* version;
    uint32_t category;
    uint32_t num_inputs;
    uint32_t num_outputs;
    bool supports_sidechain;
    uint32_t latency_samples;
} CrystalPluginDescriptorC;

// C-compatible Audio Buffer
typedef struct {
    float* left_channel;
    float* right_channel;
    const float* sidechain_channel;
    uint32_t num_samples;
} CrystalAudioBufferC;

// Function Table to be exported by any Shared Object (.so / .dll / .dylib)
typedef struct {
    CrystalPluginDescriptorC (*get_descriptor)(void);
    void* (*create_instance)(float sample_rate);
    void (*destroy_instance)(void* instance);
    void (*process_audio)(void* instance, CrystalAudioBufferC* buffer);
    void (*reset_state)(void* instance);
    uint32_t (*get_parameter_count)(void* instance);
    float (*get_parameter_value)(void* instance, uint32_t param_index);
    void (*set_parameter_value)(void* instance, uint32_t param_index, float value);
    const char* (*get_parameter_name)(void* instance, uint32_t param_index);

    // Omid Protocol (Object-MIDI) Integration
    void (*handle_omid_packet)(void* instance, const uint8_t packet_bytes[8]);
    bool (*poll_omid_response)(void* instance, uint8_t out_packet_bytes[8]);
} CrystalPluginVTable;

// Entry point required by every Crystal-Wave plugin shared library:
// extern CrystalPluginVTable crystal_plugin_get_vtable(void);

#ifdef __cplusplus
}
#endif

#endif // CRYSTAL_WAVE_PLUGIN_H
