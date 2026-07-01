#ifndef COREVM_GUEST_H
#define COREVM_GUEST_H

#include <assert.h>
#include <stddef.h>
#include <stdint.h>

#include "polkavm_guest.h"

#define POLKAVM_REGS_FOR_TY_uint64x2 2

typedef struct { uint64_t _1; uint64_t _2; } uint64x2;

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
POLKAVM_IMPORT(void, corevm_yield_video_frame_ext, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_audio_mode_ext, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_yield_audio_samples_ext, uint64_t, uint64_t);
POLKAVM_IMPORT(uint64_t, corevm_recv_message_ext, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(void, corevm_send_message_ext, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(uint64_t, corevm_read_console_data_ext, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(uint64x2, corevm_video_input_mode_ext);
POLKAVM_IMPORT(uint64_t, corevm_read_video_frame_ext, uint64_t, uint64_t, uint64_t, uint64_t);
POLKAVM_IMPORT(uint64_t, corevm_audio_input_mode_ext);
POLKAVM_IMPORT(uint64_t, corevm_read_audio_samples_ext, uint64_t, uint64_t, uint64_t);

// Convenience wrappers.

typedef uint64_t UnsignedGas;
typedef int64_t SignedGas;
typedef uint32_t ServiceId;

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
    COREVM_STDOUT = 1,
    COREVM_STDERR = 2
};

inline static void corevm_yield_console_data(enum CoreVmConsoleStream stream, const void* data, size_t size) {
    corevm_yield_console_data_ext(stream, (uintptr_t) data, size);
}

enum CoreVmVideoFrameFormat {
    COREVM_VIDEO_RGB88_INDEXED8 = 1
};

enum CoreVmInputVideoFrameFormat {
    COREVM_INPUT_VIDEO_RGB888 = 1
};

#define COREVM_VIDEO_MODE_QUANTIZATION_LEVEL(level) ((level) & UINT64_C(15))
#define COREVM_VIDEO_MODE_CHROMA_SUBSAMPLING (UINT64_C(1) << 4)
#define COREVM_VIDEO_MODE_RAW (UINT64_C(1) << 5)

struct CoreVmVideoMode {
    uint64_t options;
    uint16_t width;
    uint16_t height;
    uint16_t refresh_rate;
};

inline static void corevm_video_mode(const struct CoreVmVideoMode* mode) {
    corevm_video_mode_ext(mode->width, mode->height, mode->refresh_rate, mode->options);
}

inline static void corevm_yield_video_frame(
    const void* data,
    size_t size,
    enum CoreVmVideoFrameFormat format
) {
    corevm_yield_video_frame_ext((uintptr_t) data, size, format);
}

enum CoreVmAudioSampleFormat {
    COREVM_AUDIO_S16LE = 1,
    COREVM_AUDIO_S32LE = 2,
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

#define COREVM_BUFFER_IS_TOO_SMALL (-1)
#define COREVM_NONE (-2)

inline static size_t corevm_try_recv_message_into(void* buf, size_t buf_size) {
    size_t len = corevm_recv_message_ext((uintptr_t) buf, buf_size, 1);
    if (len == (uint64_t)-1) {
        return (size_t) COREVM_NONE;
    }
    if (len > buf_size) {
        return (size_t) COREVM_BUFFER_IS_TOO_SMALL;
    }
    return len;
}

inline static size_t corevm_recv_message_into(void* buf, size_t buf_size) {
    size_t len = corevm_recv_message_ext((uintptr_t) buf, buf_size, 0);
    if (len > buf_size) {
        return (size_t) COREVM_BUFFER_IS_TOO_SMALL;
    }
    return len;
}

inline static void corevm_send_message(ServiceId service_id, const void* data, size_t size) {
    corevm_send_message_ext(service_id, (uintptr_t) data, size);
}

inline static size_t corevm_try_read_console_data_into(void* buf, size_t buf_size) {
    size_t len = corevm_read_console_data_ext((uintptr_t) buf, buf_size, 1);
    if (len == (uint64_t)-1) {
        return (size_t) COREVM_NONE;
    }
    if (len > buf_size) {
        return (size_t) COREVM_BUFFER_IS_TOO_SMALL;
    }
    return len;
}

inline static size_t corevm_read_console_data_into(void* buf, size_t size) {
    return corevm_read_console_data_ext((uintptr_t) buf, size, 0);
}

inline static struct CoreVmVideoMode corevm_video_input_mode() {
    uint64x2 t = corevm_video_input_mode_ext();
    return (struct CoreVmVideoMode) {
        .width = t._1 & 0xffff,
        .height = (t._1 >> 16) & 0xffff,
        .refresh_rate = (t._1 >> 32) & 0xffff,
        .options = (t._1 >> 48) & 0x3f,
    };
}

inline static size_t corevm_read_video_frame_into(
    void* buf,
    size_t buf_size,
    enum CoreVmInputVideoFrameFormat format
) {
    size_t len = corevm_read_video_frame_ext((uintptr_t) buf, buf_size, format, 0);
    if (len > buf_size) {
        return (size_t) COREVM_BUFFER_IS_TOO_SMALL;
    }
    return len;
}

inline static struct CoreVmAudioMode corevm_audio_input_mode() {
    uint64_t w0 = corevm_audio_input_mode_ext();
    return (struct CoreVmAudioMode) {
        .sample_rate = w0 & 0xffffffff,
        .channels = (w0 >> 32) & 0xff,
        .sample_format = (w0 >> 40) & 0xff,
    };
}

inline static size_t corevm_read_audio_samples_into(void* buf, size_t buf_size) {
    size_t len = corevm_read_audio_samples_ext((uintptr_t) buf, buf_size, 0);
    if (len > buf_size) {
        return (size_t) COREVM_BUFFER_IS_TOO_SMALL;
    }
    return len;
}

#endif
