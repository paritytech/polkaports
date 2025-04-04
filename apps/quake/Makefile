CC = polkavm-cc
CFLAGS = -Os -std=gnu99 -Wno-dangling-else -Wno-trigraphs -Wno-unused-value -Wno-pointer-to-int-cast -Wno-pointer-sign -flto -ferror-limit=0 -ggdb
LDFLAGS =
OUTPUT = output

QUAKE_OBJS = \
	$(OUTPUT)/cd_null.o \
	$(OUTPUT)/cl_demo.o \
	$(OUTPUT)/cl_effects.o \
	$(OUTPUT)/cl_input.o \
	$(OUTPUT)/cl_main.o \
	$(OUTPUT)/cl_parse.o \
	$(OUTPUT)/cl_screen.o \
	$(OUTPUT)/cl_tent.o \
	$(OUTPUT)/cl_view.o \
	$(OUTPUT)/cmd.o \
	$(OUTPUT)/cmodel.o \
	$(OUTPUT)/common.o \
	$(OUTPUT)/console.o \
	$(OUTPUT)/crc.o \
	$(OUTPUT)/cvar.o \
	$(OUTPUT)/d_edge.o \
	$(OUTPUT)/d_init.o \
	$(OUTPUT)/d_modech.o \
	$(OUTPUT)/d_part.o \
	$(OUTPUT)/d_polyse.o \
	$(OUTPUT)/d_scan.o \
	$(OUTPUT)/d_sky.o \
	$(OUTPUT)/d_sprite.o \
	$(OUTPUT)/d_surf.o \
	$(OUTPUT)/d_vars.o \
	$(OUTPUT)/host.o \
	$(OUTPUT)/host_cmd.o \
	$(OUTPUT)/in_null.o \
	$(OUTPUT)/keys.o \
	$(OUTPUT)/mathlib.o \
	$(OUTPUT)/menu.o \
	$(OUTPUT)/net_loop.o \
	$(OUTPUT)/net_vcr.o \
	$(OUTPUT)/net_main.o \
	$(OUTPUT)/net_win.o \
	$(OUTPUT)/pcx.o \
	$(OUTPUT)/r_aclip.o \
	$(OUTPUT)/r_alias.o \
	$(OUTPUT)/r_bsp.o \
	$(OUTPUT)/r_draw.o \
	$(OUTPUT)/r_edge.o \
	$(OUTPUT)/r_light.o \
	$(OUTPUT)/r_main.o \
	$(OUTPUT)/r_misc.o \
	$(OUTPUT)/r_model.o \
	$(OUTPUT)/r_rast.o \
	$(OUTPUT)/r_sky.o \
	$(OUTPUT)/r_sprite.o \
	$(OUTPUT)/r_surf.o \
	$(OUTPUT)/r_vars.o \
	$(OUTPUT)/snd_mem.o \
	$(OUTPUT)/snd_dma.o \
	$(OUTPUT)/sys_null.o \
	$(OUTPUT)/vid_polkavm.o \
	$(OUTPUT)/wad.o \
	$(OUTPUT)/zone.o \
	$(OUTPUT)/sv_main.o \
	$(OUTPUT)/sv_move.o \
	$(OUTPUT)/sv_phys.o \
	$(OUTPUT)/sv_user.o \
	$(OUTPUT)/sv_world.o \
	$(OUTPUT)/pr_cmds.o \
	$(OUTPUT)/pr_edict.o \
	$(OUTPUT)/pr_exec.o

$(OUTPUT)/quake.polkavm : $(OUTPUT)/quake.elf
	polkatool link $(OUTPUT)/quake --output $(OUTPUT)/quake.polkavm --min-stack-size 524288

$(OUTPUT)/quake.elf : $(QUAKE_OBJS)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $(QUAKE_OBJS)

$(OUTPUT)/%.o: %.c
	@mkdir -p output
	$(CC) -c $(CFLAGS) $*.c -o $@

clean:
	rm -rf output
