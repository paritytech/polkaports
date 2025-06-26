//! Definitions from Musl libc.

#![allow(unused)]

pub const AT_FDCWD: i32 = -100_i32;
pub const AT_PAGESZ: u64 = 6;

pub const EACCES: u64 = 13;
pub const EBADF: u64 = 9;
pub const EFAULT: u64 = 14;
pub const ENOTDIR: u64 = 20;
pub const EINVAL: u64 = 22;
pub const EIO: u64 = 5;
pub const ENOENT: u64 = 2;
pub const ENOSYS: u64 = 38;
pub const EISDIR: u64 = 21;
pub const ERANGE: u64 = 34;

pub const FILENO_STDERR: u32 = 2;
pub const FILENO_STDOUT: u32 = 1;

pub const IOV_MAX: u64 = 1024;

pub const O_RDWR: u64 = 2;
pub const O_WRONLY: u64 = 1;
pub const O_CLOEXEC: u64 = 0o2000000;
pub const O_DIRECTORY: u64 = 0o40000;

pub const PATH_MAX: u64 = 4096;

pub const SEEK_CUR: u64 = 1;
pub const SEEK_END: u64 = 2;
pub const SEEK_SET: u64 = 0;

pub const F_SETFD: u64 = 2;

pub const FD_CLOEXEC: u64 = 1;

// See `arch/riscv64/bits/syscall.h.in` for the actual values.
pub const SYS_FCNTL: u64 = 25;
pub const SYS_CLOSE: u64 = 57;
pub const SYS_EXIT: u64 = 93;
pub const SYS_EXIT_GROUP: u64 = 94;
pub const SYS_LSEEK: u64 = 62;
pub const SYS_OPENAT: u64 = 56;
pub const SYS_READ: u64 = 63;
pub const SYS_READV: u64 = 65;
pub const SYS_WRITE: u64 = 64;
pub const SYS_WRITEV: u64 = 66;
pub const SYS_SET_TID_ADDRESS: u64 = 96;
pub const SYS_IOCTL: u64 = 29;
pub const SYS_GETUID: u64 = 174;
pub const SYS_GETEUID: u64 = 175;
pub const SYS_GETGID: u64 = 176;
pub const SYS_GETEGID: u64 = 177;
pub const SYS_SETUID: u64 = 146;
pub const SYS_SETGID: u64 = 144;
pub const SYS_UNAME: u64 = 160;
pub const SYS_NEWFSTATAT: u64 = 79;
pub const SYS_CLOCK_GETTIME: u64 = 113;
pub const SYS_GETDENTS64: u64 = 61;
pub const SYS_FACCESSAT: u64 = 48;
pub const SYS_GETGROUPS: u64 = 158;
pub const SYS_SYNC: u64 = 81;
pub const SYS_DUP3: u64 = 24;
pub const SYS_GETCWD: u64 = 17;

pub const TIOCGWINSZ: u64 = 0x5413;

pub type DevT = u64;
pub type InoT = u64;
pub type ModeT = u32;
pub type NlinkT = u32;
pub type UidT = u32;
pub type GidT = u32;
pub type OffT = i64;
pub type BlksizeT = i32;
pub type BlkcntT = i64;
pub type ClockidT = i32;

#[repr(C)]
#[derive(Debug, Default)]
pub struct Timespec {
	pub tv_sec: i64,
	pub tv_nsec: i64,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Stat {
	pub st_dev: DevT,
	pub st_ino: InoT,
	pub st_mode: ModeT,
	pub st_nlink: NlinkT,
	pub st_uid: UidT,
	pub st_gid: GidT,
	pub st_rdev: DevT,
	pub __pad: u64,
	pub st_size: OffT,
	pub st_blksize: BlksizeT,
	pub __pad2: i32,
	pub st_blocks: BlkcntT,
	pub st_atim: Timespec,
	pub st_mtim: Timespec,
	pub st_ctim: Timespec,
	pub __unused: [u32; 2],
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct WinSize {
	pub row: u16,
	pub col: u16,
	pub xpixel: u16,
	pub ypixel: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct Utsname {
	pub sysname: [u8; 65],
	pub nodename: [u8; 65],
	pub release: [u8; 65],
	pub version: [u8; 65],
	pub machine: [u8; 65],
	pub domainname: [u8; 65],
}

macro_rules! utsname_field {
	($value: expr) => {{
		let value = $value;
		let mut array = [0_u8; 65];
		array[..value.len()].copy_from_slice(value);
		array
	}};
}

pub(crate) use utsname_field;

pub const fn errno(error: u64) -> u64 {
	(-(error as i64)) as u64
}
