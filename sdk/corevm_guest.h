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
POLKAVM_IMPORT(uint64_t, corevm_yield_video_frame_impl, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_video_mode_impl, uint64_t, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_audio_mode_impl, uint64_t, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(uint64_t, corevm_yield_audio_frame_impl, uint64_t, uint64_t, uint64_t);

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
            while (true) { \
                uint64_t ret = corevm_yield_console_data(stream, (uint64_t)buffer, (uint64_t)(n + 1)); \
                if (ret == 0) { \
                    break; \
                } \
            } \
        } \
    }

#define corevm_printf(format, ...) corevm_printf_impl(1, format, ##__VA_ARGS__)
#define corevm_eprintf(format, ...) corevm_printf_impl(2, format, ##__VA_ARGS__)

inline static void corevm_yield_video_frame(size_t frame_number, const void* frame, size_t frame_len) {
    while (1) {
        uint64_t ret = corevm_yield_video_frame_impl((uint64_t) frame_number, (uint64_t) frame, (uint64_t) frame_len);
        if (ret == 0) {
            break;
        }
    }
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

enum CoreVmAudioFrameFormat {
    COREVM_AUDIO_S16LE = 1
};

struct CoreVmAudioMode {
    uint32_t channels;
    uint32_t bits_per_sample;
    uint16_t sample_rate;
    enum CoreVmAudioFrameFormat format;
};

inline static void corevm_audio_mode(const struct CoreVmAudioMode* mode) {
    corevm_audio_mode_impl(
        (uint64_t) mode->channels,
        (uint64_t) mode->bits_per_sample,
        (uint64_t) mode->sample_rate,
        (uint64_t) mode->format
    );
}

inline static void corevm_yield_audio_frame(size_t frame_number, const void* frame, size_t frame_len) {
    while (1) {
        uint64_t ret = corevm_yield_audio_frame_impl((uint64_t) frame_number, (uint64_t) frame, (uint64_t) frame_len);
        if (ret == 0) {
            break;
        }
    }
}

#endif
