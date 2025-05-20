//! Definitions from Musl libc.

#![allow(unused)]

pub const AT_FDCWD: u64 = (-100_i64) as u64;
pub const AT_PAGESZ: u64 = 6;

pub const EACCES: u64 = 13;
pub const EBADF: u64 = 9;
pub const EFAULT: u64 = 14;
pub const EINVAL: u64 = 22;
pub const EIO: u64 = 5;
pub const ENOENT: u64 = 2;
pub const ENOSYS: u64 = 38;

pub const FILENO_STDERR: u64 = 2;
pub const FILENO_STDOUT: u64 = 1;

pub const IOV_MAX: u64 = 1024;

pub const O_RDWR: u64 = 2;
pub const O_WRONLY: u64 = 1;

pub const PATH_MAX: u64 = 4096;

pub const SEEK_CUR: u64 = 1;
pub const SEEK_END: u64 = 2;
pub const SEEK_SET: u64 = 0;

// See `arch/riscv64/bits/syscall.h.in` for the actual values.
pub const SYS_CLOSE: u64 = 57;
pub const SYS_EXIT: u64 = 93;
pub const SYS_LSEEK: u64 = 62;
pub const SYS_OPENAT: u64 = 56;
pub const SYS_READ: u64 = 63;
pub const SYS_READV: u64 = 65;
pub const SYS_WRITE: u64 = 64;
pub const SYS_WRITEV: u64 = 66;
pub const SYS_SET_TID_ADDRESS: u64 = 96;
pub const SYS_IOCTL: u64 = 29;

pub const TIOCGWINSZ: u64 = 0x5413;

#[repr(C)]
pub struct WinSize {
	pub row: u16,
	pub col: u16,
	pub xpixel: u16,
	pub ypixel: u16,
}

pub const fn errno(error: u64) -> u64 {
	(-(error as i64)) as u64
}
