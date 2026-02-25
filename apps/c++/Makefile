hello.polkavm: hello.elf
	polkatool link hello.elf -o hello.polkavm

hello.elf: main.cc
	polkavm-c++ -Os -flto -fno-exceptions -fno-rtti main.cc -o hello.elf

clean:
	rm -f hello.elf hello.polkavm
