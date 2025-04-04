__asm__(
".section .sdata,\"aw\"\n"
".text\n"
".global _pvm_start\n"
".type " START ",%function\n"
"_pvm_start:\n"
"tail " START "_c"
);

#include "polkavm_guest.h"

void _pvm_start(long * p);
POLKAVM_EXPORT(void, _pvm_start, long);
