.global __cp_begin
.hidden __cp_begin
.global __cp_end
.hidden __cp_end
.global __cp_cancel
.hidden __cp_cancel
.hidden __cancel
.global __syscall_cp_asm
.hidden __syscall_cp_asm
.type __syscall_cp_asm, %function
__syscall_cp_asm:
__cp_begin:
	lw t0, 0(a0)
	bnez t0, __cp_cancel

	j _syscall_polkavm
__cp_end:
	ret
__cp_cancel:
	tail __cancel
