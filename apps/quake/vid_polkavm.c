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

#include <polkavm_guest.h>

POLKAVM_IMPORT(void, pvm_set_palette, long)
POLKAVM_IMPORT(void, pvm_display, long, long, long)
POLKAVM_IMPORT(long, pvm_fetch_inputs, long, long)
POLKAVM_IMPORT(long, pvm_init_audio, long, long, long)
POLKAVM_IMPORT(long, pvm_output_audio, long, long)

extern viddef_t vid; // global video state

#define SAMPLES 256
#define CHANNELS 2
#define SAMPLE_RATE 11025

#define	BASEWIDTH	320
#define	BASEHEIGHT	200

#define FRAMES_PER_SEC_F 60.0

byte	vid_buffer[BASEWIDTH*BASEHEIGHT];
short	zbuffer[BASEWIDTH*BASEHEIGHT];
byte	surfcache[256*1024];

unsigned short	d_8to16table[256];
unsigned	d_8to24table[256];

void	VID_SetPalette (unsigned char *palette)
{
    pvm_set_palette((long)palette);
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
	vid.buffer = vid_buffer;
	vid.rowbytes = BASEWIDTH;
	
	d_pzbuffer = zbuffer;
	D_InitCaches (surfcache, sizeof(surfcache));

    VID_SetPalette(palette);
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
    pvm_display(vid.width, vid.height, (long)vid.buffer);
    s_timestamp += (1.0 / FRAMES_PER_SEC_F);

    while (1) {
        struct Event events[32];
        long count = pvm_fetch_inputs((long)&events[0], 32);
        if (count == 0) {
            break;
        }

        for (long i = 0; i < count; ++i) {
            unsigned char key = 0;
            switch (events[i].key) {
                case 0x80 + 0: key = K_UPARROW; break;
                case 0x80 + 1: key = K_DOWNARROW; break;
                case 0x80 + 2: key = K_RIGHTARROW; break;
                case 0x80 + 3: key = K_LEFTARROW; break;
                case 0x80 + 4: key = K_F1; break;
                case 0x80 + 5: key = K_F2; break;
                case 0x80 + 6: key = K_F3; break;
                case 0x80 + 7: key = K_F4; break;
                case 0x80 + 8: key = K_F5; break;
                case 0x80 + 9: key = K_F6; break;
                case 0x80 + 10: key = K_F7; break;
                case 0x80 + 11: key = K_F8; break;
                case 0x80 + 12: key = K_F9; break;
                case 0x80 + 13: key = K_F10; break;
                case 0x80 + 14: key = K_F11; break;
                case 0x80 + 15: key = K_F12; break;
                case 0x80 + 16: key = K_CAPSLOCK; break;
                case 0x80 + 17: break;
                case 0x80 + 18: break;
                case 0x80 + 19: key = K_PAUSE; break;
                case 0x80 + 20: key = K_INS; break;
                case 0x80 + 21: key = K_DEL; break;
                case 0x80 + 22: key = K_HOME; break;
                case 0x80 + 23: key = K_END; break;
                case 0x80 + 24: key = K_PGUP; break;
                case 0x80 + 25: key = K_PGDN; break;
                case 0x80 + 26: key = K_SHIFT; break;
                case 0x80 + 27: key = K_SHIFT; break;
                case 0x80 + 28: key = K_CTRL; break;
                case 0x80 + 29: key = K_CTRL; break;
                case 0x80 + 30: key = K_ALT; break;
                case 0x80 + 31: key = K_ALT; break;
                case 0x80 + 32: key = K_MOUSE1; break;
                case 0x80 + 33: key = K_MOUSE2; break;
                case 0x80 + 34: key = K_MOUSE3; break;
                case 0x80 + 35: {
                    if (!cls.demoplayback) {
                        s_mouse_x += ((signed char)events[i].value) * MOUSE_SENSITIVITY_X;
                    }
                    continue;
                }
                case 0x80 + 36: {
                    if (!cls.demoplayback) {
                        s_mouse_y += ((signed char)events[i].value) * MOUSE_SENSITIVITY_Y;
                    }
                    continue;
                }
                case 0x80 + 37: key = K_MWHEELUP; break;
                case 0x80 + 38: key = K_MWHEELDOWN; break;
                case '\n': key = K_ENTER; break;
                case 0x08: key = K_BACKSPACE; break;
                default:
                    key = events[i].key;
                    break;
            }

            if (key != 0) {
                Key_Event (key, events[i].value);
            }
        }

        if (count < 32) {
            break;
        }
    }

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
    shm->soundalive = pvm_init_audio(CHANNELS, 16, SAMPLE_RATE);
    shm->splitbuffer = false;
    shm->samplepos = 0;
    shm->submission_chunk = 1;
    shm->samples = SAMPLES / (shm->samplebits / 8);

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

        pvm_output_audio((long)buffer, count);
        paintedtime += count;
    }
}

void SND_InitScaletable (void)
{
}
