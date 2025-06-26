#ifndef COREVM_GUEST_H
#define COREVM_GUEST_H

#include <assert.h>
#include <stddef.h>
#include <stdint.h>

#include "polkavm_guest.h"

// Sanity checks.
static_assert(sizeof(size_t) <= sizeof(uint64_t), "`size_t` is too large");
static_assert(sizeof(uintptr_t) <= sizeof(uint64_t), "`uintptr_t` is too large");
static_assert(sizeof(void*) <= sizeof(uint64_t), "`void*` is too large");

// Imported functions.
POLKAVM_IMPORT(uint64_t, corevm_gas_ext);
POLKAVM_IMPORT(uint64_t, corevm_alloc_ext, uint64_t);
POLKAVM_IMPORT(void, corevm_free_ext, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_yield_console_data_ext, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_video_mode_ext, uint64_t, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_yield_video_frame_ext, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_audio_mode_ext, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_yield_audio_samples_ext, uint64_t, uint64_t);

// Convenience wrappers.

typedef uint64_t UnsignedGas;
typedef int64_t SignedGas;

inline static UnsignedGas corevm_gas() {
    return corevm_gas_ext();
}

inline static void* corevm_alloc(size_t size) {
    uintptr_t ptr = corevm_alloc_ext(size);
    return (void*) ptr;
}

inline static void corevm_free(const void* ptr, size_t size) {
    corevm_free_ext((uintptr_t) ptr, size);
}

enum CoreVmConsoleStream {
    STDOUT = 1,
    STDERR = 2
};

inline static void corevm_yield_console_data(enum CoreVmConsoleStream stream, const void* data, size_t size) {
    corevm_yield_console_data_ext(stream, (uintptr_t) data, size);
}

enum CoreVmVideoFrameFormat {
    COREVM_VIDEO_RGB88_INDEXED8 = 1
};

struct CoreVmVideoMode {
    uint32_t width;
    uint32_t height;
    uint16_t refresh_rate;
    enum CoreVmVideoFrameFormat format;
};

inline static void corevm_video_mode(const struct CoreVmVideoMode* mode) {
    corevm_video_mode_ext(mode->width, mode->height, mode->refresh_rate, mode->format);
}

inline static void corevm_yield_video_frame(const void* data, size_t size) {
    corevm_yield_video_frame_ext((uintptr_t) data, size);
}

enum CoreVmAudioSampleFormat {
    COREVM_AUDIO_S16LE = 1
};

struct CoreVmAudioMode {
    uint32_t sample_rate;
    uint8_t channels;
    enum CoreVmAudioSampleFormat sample_format;
};

inline static void corevm_audio_mode(const struct CoreVmAudioMode* mode) {
    corevm_audio_mode_ext(mode->channels, mode->sample_rate, mode->sample_format);
}

inline static void corevm_yield_audio_samples(const void* data, size_t size) {
    corevm_yield_audio_samples_ext((uintptr_t) data, size);
}

#endif
