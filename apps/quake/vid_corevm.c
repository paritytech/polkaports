/*
Copyright (C) 1996-1997 Id Software, Inc.

This program is free software; you can redistribute it and/or
modify it under the terms of the GNU General Public License
as published by the Free Software Foundation; either version 2
of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  

See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA  02111-1307, USA.

*/

#include "quakedef.h"
#include "d_local.h"
#include "sound.h"
#include "client.h"

#include <corevm_guest.h>

extern viddef_t vid; // global video state

#define SAMPLES 256
#define CHANNELS 2
// This value somehow can't be arbitrary.
#define SAMPLE_RATE 11025

#define	BASEWIDTH	320
#define	BASEHEIGHT	200

#define FRAMES_PER_SEC 60
#define FRAMES_PER_SEC_F 60.0

#define PALETTE_LEN (256 * 3)
#define FRAME_LEN (BASEWIDTH * BASEHEIGHT)

byte	vid_buffer[1 + PALETTE_LEN + FRAME_LEN];
short	zbuffer[FRAME_LEN];
byte	surfcache[256*1024];

unsigned short	d_8to16table[256];
unsigned	d_8to24table[256];

void	VID_SetPalette (unsigned char *palette)
{
    vid_buffer[0] = 1;
    memcpy(vid_buffer + 1, palette, PALETTE_LEN);
}

void	VID_ShiftPalette (unsigned char *palette)
{
    VID_SetPalette(palette);
}

void	VID_Init (unsigned char *palette)
{
	vid.width = BASEWIDTH;
	vid.height = BASEHEIGHT;
	vid.aspect = 1.0;
	vid.numpages = 1;
	memcpy(vid.colormap, host_colormap, 16384);
	vid.fullbright = 256 - LittleLong (*((int *)vid.colormap + 2048));
	vid.buffer = vid_buffer + 1 + PALETTE_LEN;
	vid.rowbytes = BASEWIDTH;
	
	d_pzbuffer = zbuffer;
	D_InitCaches (surfcache, sizeof(surfcache));

    VID_SetPalette(palette);
    struct CoreVmVideoMode mode = {
        .width = vid.width,
        .height = vid.height,
        .refresh_rate = FRAMES_PER_SEC,
        .format = COREVM_VIDEO_RGB88_INDEXED8,
    };
    corevm_video_mode(&mode);
}

void	VID_Shutdown (void)
{
}

struct Event {
    unsigned char key;
    unsigned char value;
};

static float s_timestamp = 0.0;
static float s_samples_pending = 0.0;
static float s_mouse_x = 0.0;
static float s_mouse_y = 0.0;
static const float MOUSE_SENSITIVITY_X = 0.17;
static const float MOUSE_SENSITIVITY_Y = 0.15;

void	VID_Update (vrect_t *rects)
{
    corevm_yield_video_frame(vid_buffer, 1 + PALETTE_LEN + FRAME_LEN);
    s_timestamp += (1.0 / FRAMES_PER_SEC_F);

    if (cls.demoplayback) {
        return;
    }

    int dx = (int)s_mouse_x;
    int dy = (int)s_mouse_y;

    if (dx == 0 && dy == 0) {
        return;
    }

    s_mouse_x -= dx;
    s_mouse_y -= dy;

    cl.viewangles[YAW] -= dx;
    cl.viewangles[PITCH] += dy;

    CL_StopPitchDrift ();

    if (cl.viewangles[PITCH] > 80)
        cl.viewangles[PITCH] = 80;
    if (cl.viewangles[PITCH] < -70)
        cl.viewangles[PITCH] = -70;

    if (cl.viewangles[ROLL] > 50)
        cl.viewangles[ROLL] = 50;
    if (cl.viewangles[ROLL] < -50)
        cl.viewangles[ROLL] = -50;
}

qboolean SNDDMA_Init(void) {
    memset ((void *)&sn, 0, sizeof (sn));
    shm = &sn;
    shm->channels = CHANNELS;
    shm->samplebits = 16;
    shm->speed = SAMPLE_RATE;
    shm->soundalive = 1;
    shm->splitbuffer = false;
    shm->samplepos = 0;
    shm->submission_chunk = 1;
    shm->samples = SAMPLES / (shm->samplebits / 8);
    struct CoreVmAudioMode mode = {
        .channels = shm->channels,
        .sample_rate = SAMPLE_RATE,
        .sample_format = COREVM_AUDIO_S16LE,
    };
    corevm_audio_mode(&mode);
    return 1;
}

double Sys_FloatTime(void)
{
    return s_timestamp;
}

void S_RenderSoundFrame(void)
{
    if (!shm) {
        return;
    }

    s_samples_pending += (1.0 / FRAMES_PER_SEC_F) * shm->speed;

    short buffer[SAMPLES * CHANNELS];
    for (;;) {
        long count = ((long)s_samples_pending);
        if (count <= 0) {
            break;
        }

        if (count > SAMPLES) {
            count = SAMPLES;
        }

        s_samples_pending -= count;
        memset(buffer, 0, count * sizeof(unsigned short) * CHANNELS);

        for (channel_t * ch = channels; ch < (channels + total_channels); ch++) {
            if (!ch->sfx || (!ch->leftvol && !ch->rightvol)) {
                continue;
            }

            sfxcache_t * sc = S_LoadSound (ch->sfx);
            if (!sc) {
                continue;
            }

            long offset = 0;
            long now = paintedtime;
            long end_time = now + count;
            while (now < end_time) {
                long local_count = end_time - now;
                if (ch->end < end_time) {
                    local_count = ch->end - now;
                }

                if (local_count > 0) {
                    if (sc->width == 1) {
                        if (ch->leftvol > 255) {
                            ch->leftvol = 255;
                        }
                        if (ch->rightvol > 255) {
                            ch->rightvol = 255;
                        }

                        long left_vol = ch->leftvol;
                        long right_vol = ch->rightvol;

                        if (sc->stereo) {
                            unsigned char * sfx = (unsigned char *)sc->data + ch->pos * 2;
                            printf("TODO: unimplemented: 8-bit stereo sound mixing\n");
                        } else {
                            unsigned char * sfx = (unsigned char *)sc->data + ch->pos;
                            for (long i = 0; i < local_count; ++i) {
                                unsigned char sample = *sfx++;
                                short sample16 = (((short)sample) - 128);
                                int left = buffer[(offset + i) * CHANNELS];
                                left += sample16 * left_vol;

                                if (left > 32767) {
                                    left = 32767;
                                }
                                if (left < -32768) {
                                    left = -32768;
                                }

                                buffer[(offset + i) * CHANNELS] = left;

                                if (CHANNELS == 2) {
                                    int right = buffer[(offset + i) * CHANNELS + 1];
                                    right += sample16 * right_vol;
                                    if (right > 32767) {
                                        right = 32767;
                                    }
                                    if (right < -32768) {
                                        right = -32768;
                                    }

                                    buffer[(offset + i) * CHANNELS + 1] = right;
                                }
                            }
                        }
                    } else {
                        printf("TODO: unimplemented: 16-bit sound mixing\n");
                    }

                    ch->pos += local_count;
                    now += local_count;
                    offset += local_count;
                }

                if (now >= ch->end) {
                    if (sc->loopstart >= 0) {
                        ch->pos = sc->loopstart;
                        ch->end = now + sc->length - ch->pos;
                    } else {
                        ch->sfx = NULL;
                        break;
                    }
                }
            }
        }

        corevm_yield_audio_frame(buffer, count*sizeof(int16_t)*CHANNELS);
        paintedtime += count;
    }
}

void SND_InitScaletable (void)
{
}
