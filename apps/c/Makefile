hello.polkavm: hello.elf
	polkatool link hello.elf -o hello.polkavm

hello.elf: hello.c
	polkavm-cc -flto -Os hello.c -o hello.elf

clean:
	rm -f hello.elf hello.polkavm
