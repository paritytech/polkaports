#ifndef COREVM_GUEST_H
#define COREVM_GUEST_H

#include <assert.h>
#include <stdint.h>
#include <stdio.h>

#include "polkavm_guest.h"

// Sanity checks.
static_assert(sizeof(size_t) <= sizeof(uint64_t), "`size_t` is too large");
static_assert(sizeof(void*) <= sizeof(uint64_t), "`void*` is too large");

POLKAVM_IMPORT(uint64_t, corevm_gas);
POLKAVM_IMPORT(uint64_t, corevm_alloc, uint64_t);
POLKAVM_IMPORT(void, corevm_free, uint64_t, uint64_t);
POLKAVM_IMPORT(uint64_t, corevm_yield_console_data, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(uint64_t, corevm_yield_video_frame_impl, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_video_mode_impl, uint64_t, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_audio_mode_impl, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(uint64_t, corevm_yield_audio_frame_impl, uint64_t, uint64_t);

#ifndef COREVM_PRINTF_BUFFER_LEN
#define COREVM_PRINTF_BUFFER_LEN 4096
#endif

#define corevm_printf_impl(stream, format, ...) \
    { \
        char buffer[COREVM_PRINTF_BUFFER_LEN]; \
        int n = snprintf(buffer, COREVM_PRINTF_BUFFER_LEN, format, ##__VA_ARGS__); \
        if (n > 0) { \
            if (n == COREVM_PRINTF_BUFFER_LEN) { \
                n = COREVM_PRINTF_BUFFER_LEN - 1; \
            } \
            buffer[n] = 0; \
            corevm_yield_console_data(stream, (uint64_t)buffer, (uint64_t)(n + 1)); \
        } \
    }

#define corevm_printf(format, ...) corevm_printf_impl(1, format, ##__VA_ARGS__)
#define corevm_eprintf(format, ...) corevm_printf_impl(2, format, ##__VA_ARGS__)

inline static void corevm_yield_video_frame(const void* frame, size_t frame_len) {
    corevm_yield_video_frame_impl((uint64_t) frame, (uint64_t) frame_len);
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
    corevm_video_mode_impl(
        (uint64_t) mode->width,
        (uint64_t) mode->height,
        (uint64_t) mode->refresh_rate,
        (uint64_t) mode->format
    );
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
    corevm_audio_mode_impl(
        (uint64_t) mode->channels,
        (uint64_t) mode->sample_rate,
        (uint64_t) mode->sample_format
    );
}

inline static void corevm_yield_audio_frame(const void* frame, size_t frame_len) {
    corevm_yield_audio_frame_impl((uint64_t) frame, (uint64_t) frame_len);
}

#endif
