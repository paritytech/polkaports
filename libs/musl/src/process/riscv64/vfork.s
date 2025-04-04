.global vfork
.type vfork,@function
vfork:
	unimp
	.hidden __syscall_ret
	j __syscall_ret
