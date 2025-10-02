use alloc::{collections::BTreeMap, vec};
use core::{ffi::CStr, mem::size_of};

use crate::{
	debug, libc::*, Environment, Error, FileSystem, IntoSyscallRet, Machine, MachineError, Reg,
	SeekFrom,
};

use SyscallOutcome::*;

/// Serializable kernel state.
pub struct KernelState<Fd> {
	pub fds: BTreeMap<u32, Fd>,
}

impl<Fd> KernelState<Fd> {
	pub fn new() -> Self {
		Self { fds: BTreeMap::new() }
	}
}

impl<Fd> Default for KernelState<Fd> {
	fn default() -> Self {
		Self::new()
	}
}

/// Linux kernel engine that implements system calls.
pub struct Kernel<C: Machine + Environment + FileSystem> {
	/// The execution context of all syscalls.
	pub context: C,
	/// Persistent state.
	pub state: KernelState<C::Fd>,
	uid: u32,
	gid: u32,
	max_fd: Option<u32>,
}

impl<C: Machine + Environment + FileSystem> Kernel<C> {
	pub fn new(context: C, state: KernelState<C::Fd>) -> Self {
		Self { context, state, uid: 0, gid: 0, max_fd: None }
	}

	pub fn handle_syscall(&mut self) -> Result<SyscallOutcome, MachineError> {
		let syscall = self.context.reg(Reg::A0);
		let a1 = self.context.reg(Reg::A1);
		let a2 = self.context.reg(Reg::A2);
		let a3 = self.context.reg(Reg::A3);
		let a4 = self.context.reg(Reg::A4);
		let a5 = self.context.reg(Reg::A5);
		match syscall {
			SYS_READ => {
				let result = self.handle_read(a1 as i64 as i32, a2, a3);
				log::debug!("Syscall read(fd={a1}, address={a2:#x}, length={a3}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_READV => {
				let result = self.handle_readv(a1 as i64 as i32, a2, a3);
				log::debug!("Syscall readv(fd={a1}, iov={a2:#x}, iovcnt={a3}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_WRITE => {
				let result = self.handle_write(a1 as i64 as i32, a2, a3);
				log::debug!("Syscall write(fd={a1}, address={a2:#x}, length={a3}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_WRITEV => {
				let result = self.handle_writev(a1 as i64 as i32, a2, a3);
				log::debug!("Syscall writev(fd={a1}, iov={a2:#x}, iovcnt={a3}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_EXIT => {
				log::debug!("Syscall exit(status={a1})");
				return Ok(Exit(a1 as u8));
			},
			SYS_EXIT_GROUP => {
				log::debug!("Syscall exit_group(status={a1})");
				return Ok(Exit(a1 as u8));
			},
			SYS_TKILL => {
				if a1 == 0 && a2 != 0 {
					// Abort the program.
					log::debug!(
						"Syscall tkill(pid={a1}, signal={}) = Ok(Exit({}))",
						debug::Signal(a2 as u8),
						a2 as u8
					);
					return Ok(Exit(a2 as u8));
				}
				let result = self.handle_tkill(a1, a2);
				log::debug!("Syscall tkill(pid={a1}, signal={})", debug::Signal(a2 as u8));
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_OPENAT => {
				let result = self.handle_openat(a1, a2, a3);
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_LSEEK => {
				let result = self.handle_lseek(a1 as i64 as i32, a2 as i64, a3);
				log::debug!("Syscall lseek(fd={a1}, offset={a2}, whence={a3}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_CLOSE => {
				let result = self.handle_close(a1 as i64 as i32);
				log::debug!("Syscall close(fd={a1}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_SET_TID_ADDRESS => {
				let result = self.handle_set_tid_address(a1);
				log::debug!("Syscall set_tid_address(tid_ptr={a1:#x}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_IOCTL => {
				let result = self.handle_ioctl(a1, a2, a3);
				log::debug!("Syscall ioctl(fd={a1}, op={a2:#x}, {a3}, {a4}, {a5}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_GETUID => {
				let result = self.handle_getuid();
				log::debug!("Syscall getuid() = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_GETEUID => {
				let result = self.handle_geteuid();
				log::debug!("Syscall geteuid() = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_GETGID => {
				let result = self.handle_getgid();
				log::debug!("Syscall getgid() = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_GETEGID => {
				let result = self.handle_getegid();
				log::debug!("Syscall getegid() = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_SETUID => {
				let result = self.handle_setuid(a1);
				log::debug!("Syscall setuid({a1}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_SETGID => {
				let result = self.handle_setgid(a1);
				log::debug!("Syscall setgid({a1}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_UNAME => {
				let result = self.handle_uname(a1);
				log::debug!("Syscall uname({a1:#x}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_NEWFSTATAT => {
				let result = self.handle_newfstatat(a1, a2, a3, a4);
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_CLOCK_GETTIME => {
				let result = self.handle_clock_gettime(a1, a2);
				log::debug!("Syscall clock_gettime({a1}, {a2:#x}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_FCNTL => {
				let result = self.handle_fcntl(a1 as i64 as i32, a2, a3);
				log::debug!("Syscall fcntl(fd={a1}, op={a2}, {a3}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_GETDENTS64 => {
				let result = self.handle_getdents64(a1 as i64 as i32, a2, a3);
				log::debug!("Syscall getdents64(fd={a1}, buf={a2:#x}, size={a3}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_FACCESSAT => {
				let result = self.handle_faccessat(a1, a2, a3, a4);
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_GETGROUPS => {
				let result = self.handle_getgroups(a1, a2);
				log::debug!("Syscall getgroups(size={a1}, list={a2:#x}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_SYNC => {
				let result = Ok(());
				log::debug!("Syscall sync() = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_GETCWD => {
				let result = self.handle_getcwd(a1, a2);
				log::debug!("Syscall getcwd(buf={a1:#x}, size={a2}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_PPOLL => {
				let result = self.handle_ppoll(a1, a2, a3, a4);
				log::debug!("Syscall ppoll(fds={a1:#x}, nfds={a2}, timeout={a3:#x}, sigmask={a4:#x}) = {result:?}");
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_RT_SIGACTION => {
				let result = self.handle_rt_sigaction(a1, a2, a3, a4);
				log::debug!("Syscall rt_sigaction(signal={}, action={a2:#x}, old_action={a3:#x}, sigset_size={a4}) = {result:?}", debug::Signal(a1 as u8));
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			SYS_RT_SIGPROCMASK => {
				let result = self.handle_rt_sigprocmask(a1, a2, a3, a4);
				log::debug!(
                    "Syscall rt_sigprocmask(how={}, set={a2:#x}, old_set={a3:#x}, sigset_size={a4}) = {result:?}",
                    debug::SigMask(a1 as u8)
				);
				self.context.set_reg(Reg::A0, result.into_ret());
			},
			_ => {
				log::debug!(
					"Unimplemented syscall: {syscall:>3}, \
                    args = [0x{a1:>016x}, 0x{a2:>016x}, 0x{a3:>016x}, 0x{a4:>016x}, 0x{a5:>016x}]"
				);
				self.context.set_reg(Reg::A0, errno(ENOSYS));
			},
		}
		Ok(Continue)
	}

	fn handle_open(&mut self, path: &CStr, flags: u64) -> Result<u32, Error> {
		if (flags & (O_WRONLY | O_RDWR)) != 0 {
			return Err(Error(EACCES));
		}
		let file = self.context.open(path, flags)?;
		let max_fd = match self.max_fd {
			Some(ref mut max_fd) => max_fd,
			ref mut p @ None => p.insert(self.state.fds.keys().copied().max().unwrap_or(0)),
		};
		*max_fd += 1;
		let fd = MAX_RESERVED_FD + *max_fd;
		self.state.fds.insert(fd, file);
		Ok(fd)
	}

	fn handle_openat(&mut self, dirfd: u64, path: u64, flags: u64) -> Result<u32, Error> {
		let path = self.context.read_cstring(path, PATH_MAX)?;
		let dirfd = dirfd as i64 as i32;
		let result = self.do_handle_openat(dirfd, &path, flags);
		log::debug!(
			"Syscall openat(dirfd={}, path={path:?}, flags={flags:#o}) = {result:?}",
			debug::DirFd(dirfd)
		);
		result
	}

	#[inline]
	fn do_handle_openat(&mut self, dirfd: i32, path: &CStr, flags: u64) -> Result<u32, Error> {
		if dirfd != AT_FDCWD {
			return Err(Error(ENOSYS));
		}
		self.handle_open(path, flags)
	}

	fn handle_close(&mut self, fd: i32) -> Result<(), Error> {
		let fd = fd.try_into().map_err(|_| Error(EBADF))?;
		if self.state.fds.remove(&fd).is_some() {
			return Ok(());
		}
		Err(Error(EBADF))
	}

	fn handle_read(&mut self, fd: i32, address: u64, length: u64) -> Result<u64, Error> {
		let fd = fd.try_into().map_err(|_| Error(EBADF))?;
		let fd = self.state.fds.get_mut(&fd).ok_or(Error(EBADF))?;
		if address.checked_add(length).is_none() || u32::try_from(address + length).is_err() {
			return Err(Error(EFAULT));
		}
		let buf_len = length.try_into().map_err(|_| Error(EFAULT))?;
		let mut buf = vec![0_u8; buf_len];
		let num_bytes_read = self.context.read(fd, &mut buf)?;
		buf.resize(num_bytes_read, 0_u8);
		self.context.write_memory(address, &buf[..])?;
		Ok(num_bytes_read as u64)
	}

	fn handle_readv(&mut self, fd: i32, iov: u64, iovcnt: u64) -> Result<u64, Error> {
		if iovcnt == 0 || iovcnt > IOV_MAX {
			return Err(Error(EINVAL));
		}

		let mut total_length = 0;
		for n in 0..iovcnt {
			let address = self.context.read_u64(iov.wrapping_add(n * 16))?;
			let length = self.context.read_u64(iov.wrapping_add(n * 16).wrapping_add(8))?;
			self.handle_read(fd, address, length)?;
			total_length += length;
		}

		Ok(total_length)
	}

	fn handle_write(&mut self, fd: i32, address: u64, length: u64) -> Result<u64, Error> {
		let fd = fd.try_into().map_err(|_| Error(EBADF))?;
		if fd != FILENO_STDOUT && fd != FILENO_STDERR && !self.state.fds.contains_key(&fd) {
			return Err(Error(EBADF));
		}

		if address.checked_add(length).is_none() || u32::try_from(address + length).is_err() {
			return Err(Error(EFAULT));
		}

		let data = self.context.read_memory(address, length)?;

		match fd {
			FILENO_STDOUT => self.context.write_to_stdout(&data[..]),
			FILENO_STDERR => self.context.write_to_stderr(&data[..]),
			_ => Err(Error(ENOSYS)),
		}
	}

	fn handle_writev(&mut self, fd: i32, iov: u64, iovcnt: u64) -> Result<u64, Error> {
		if iovcnt == 0 || iovcnt > IOV_MAX {
			return Err(Error(EINVAL));
		}

		let mut total_length = 0;
		for n in 0..iovcnt {
			let address = self.context.read_u64(iov.wrapping_add(n * 16))?;
			let length = self.context.read_u64(iov.wrapping_add(n * 16).wrapping_add(8))?;
			self.handle_write(fd, address, length)?;
			total_length += length;
		}

		Ok(total_length)
	}

	fn handle_lseek(&mut self, fd: i32, offset: i64, whence: u64) -> Result<u64, Error> {
		let fd = fd.try_into().map_err(|_| Error(EBADF))?;
		let fd = self.state.fds.get_mut(&fd).ok_or(Error(EBADF))?;
		let from = match whence {
			SEEK_SET => SeekFrom::Start(offset as u64),
			SEEK_CUR => SeekFrom::Current(offset),
			SEEK_END => SeekFrom::End(offset),
			_ => {
				return Err(Error(EINVAL));
			},
		};
		self.context.seek(fd, from)
	}

	fn handle_set_tid_address(&mut self, thread_id_address: u64) -> Result<u64, Error> {
		if thread_id_address != 0 {
			self.context.write_u32(thread_id_address, THREAD_ID)?;
		}
		Ok(THREAD_ID.into())
	}

	fn handle_ioctl(&mut self, _fd: u64, op: u64, arg0: u64) -> Result<(), Error> {
		if op == TIOCGWINSZ {
			// NOTE This is a stub to make Musl's `__stdout_write` use line buffering.
			let address = arg0;
			let vt100_size = WinSize { col: 80, row: 25, xpixel: 0, ypixel: 0 };
			self.context.write_memory(address, as_u8_slice(&vt100_size))?;
			return Ok(());
		}
		Err(Error(ENOSYS))
	}

	fn handle_fcntl(&mut self, fd: i32, op: u64, arg0: u64) -> Result<(), Error> {
		let fd = fd.try_into().map_err(|_| Error(EBADF))?;
		if !self.state.fds.contains_key(&fd) {
			return Err(Error(EBADF));
		}
		match (op, arg0) {
			(F_SETFD, FD_CLOEXEC) => Ok(()),
			_ => Err(Error(ENOSYS)),
		}
	}

	fn handle_getuid(&mut self) -> Result<u32, Error> {
		Ok(self.uid)
	}

	fn handle_geteuid(&mut self) -> Result<u32, Error> {
		Ok(self.uid)
	}

	fn handle_getgid(&mut self) -> Result<u32, Error> {
		Ok(self.gid)
	}

	fn handle_getegid(&mut self) -> Result<u32, Error> {
		Ok(self.gid)
	}

	fn handle_setuid(&mut self, uid: u64) -> Result<(), Error> {
		if uid == u64::from(self.uid) {
			Ok(())
		} else {
			Err(Error(ENOSYS))
		}
	}

	fn handle_setgid(&mut self, gid: u64) -> Result<(), Error> {
		if gid == u64::from(self.gid) {
			Ok(())
		} else {
			Err(Error(ENOSYS))
		}
	}

	fn handle_getgroups(&mut self, size: u64, address: u64) -> Result<u64, Error> {
		if size == 0 {
			return Ok(1);
		}
		self.context.write_u32(address, self.gid)?;
		Ok(1)
	}

	fn handle_uname(&mut self, address: u64) -> Result<(), Error> {
		if address == 0 {
			return Err(Error(EFAULT));
		}
		let utsname = Utsname {
			// This should always equal "Linux" because some programs depend on this exact value.
			sysname: utsname_field!(b"Linux"),
			nodename: utsname_field!(b"node"),
			// Crate version.
			version: utsname_field!(concat!(
				env!("CARGO_PKG_NAME"),
				"-",
				env!("CARGO_PKG_VERSION")
			)
			.as_bytes()),
			// Linux version. See `sysroot-corevm/include/linux/version.h`.
			release: utsname_field!(b"6.15.0"),
			machine: utsname_field!(b"riscv64emac"),
			domainname: [0_u8; 65],
		};
		self.context.write_memory(address, as_u8_slice(&utsname))?;
		Ok(())
	}

	fn handle_newfstatat(
		&mut self,
		dirfd: u64,
		path_address: u64,
		stat_address: u64,
		flags: u64,
	) -> Result<(), Error> {
		let dirfd = dirfd as i64 as i32;
		let path = self.context.read_cstring(path_address, PATH_MAX)?;
		let result = self.do_handle_newfstatat(dirfd, &path, stat_address, flags);
		log::debug!(
			"Syscall newfstatat(dirfd={}, path={path:?}, stat={stat_address:#x}, flags={flags:#x}) = {result:?}",
			debug::DirFd(dirfd)
		);
		result
	}

	#[inline]
	fn do_handle_newfstatat(
		&mut self,
		dirfd: i32,
		path: &CStr,
		stat_address: u64,
		_flags: u64,
	) -> Result<(), Error> {
		if dirfd != AT_FDCWD {
			return Err(Error(ENOSYS));
		}
		let meta = self.context.metadata(path).map_err(|_| Error(ENOENT))?;
		let stat = Stat {
			st_dev: 0,
			st_ino: meta.id,
			st_rdev: 0,
			st_uid: 0,
			st_gid: 0,
			st_nlink: 1,
			st_mode: meta.mode,
			st_blksize: meta.block_size as BlksizeT,
			st_blocks: meta.size.div_ceil(512) as BlkcntT,
			st_size: meta.size as OffT,
			..Default::default()
		};
		self.context.write_memory(stat_address, as_u8_slice(&stat))?;
		Ok(())
	}

	fn handle_clock_gettime(&mut self, _clock_id: u64, address: u64) -> Result<(), Error> {
		if address == 0 {
			return Err(Error(EFAULT));
		}
		let ts = Timespec { tv_sec: 0, tv_nsec: 0 };
		self.context.write_memory(address, as_u8_slice(&ts))?;
		Ok(())
	}

	fn handle_getdents64(
		&mut self,
		fd: i32,
		buf_address: u64,
		buf_size: u64,
	) -> Result<u64, Error> {
		let fd = fd.try_into().map_err(|_| Error(EBADF))?;
		let fd = self.state.fds.get_mut(&fd).ok_or(Error(EBADF))?;
		let buf_size = buf_size as usize;
		let mut buf = vec![0_u8; buf_size];
		let mut offset = 0;
		while offset != buf_size {
			let n = self.context.read_dir(fd, &mut buf[offset..])?;
			if n == 0 {
				break;
			}
			offset += n;
		}
		self.context.write_memory(buf_address, &buf[..offset])?;
		Ok(offset as u64)
	}

	fn handle_faccessat(
		&mut self,
		dirfd: u64,
		path_address: u64,
		mode: u64,
		flags: u64,
	) -> Result<(), Error> {
		let path = self.context.read_cstring(path_address, PATH_MAX)?;
		let dirfd = dirfd as i64 as i32;
		let mode = mode as u32;
		let result = self.do_handle_faccessat(dirfd, &path, mode, flags);
		log::debug!(
			"Syscall faccessat(dirfd={}, path={path:?}, mode={mode:#o}, flags={flags:#x}) = {result:?}",
            debug::DirFd(dirfd)
		);
		result
	}

	#[inline]
	fn do_handle_faccessat(
		&mut self,
		dirfd: i32,
		path: &CStr,
		mode: u32,
		_flags: u64,
	) -> Result<(), Error> {
		if dirfd != AT_FDCWD {
			return Err(Error(ENOSYS));
		}
		let meta = self.context.metadata(path)?;
		if meta.mode & mode == mode {
			return Ok(())
		}
		Err(Error(EACCES))
	}

	fn handle_getcwd(&mut self, buf_address: u64, buf_size: u64) -> Result<u64, Error> {
		let cwd = c"/".to_bytes_with_nul();
		if buf_size < cwd.len() as u64 {
			return Err(Error(ERANGE));
		}
		self.context.write_memory(buf_address, cwd)?;
		Ok(cwd.len() as u64)
	}

	fn handle_tkill(&mut self, pid: u64, _sig: u64) -> Result<(), Error> {
		if pid != 0 {
			return Err(Error(ENOSYS));
		}
		Ok(())
	}

	fn handle_ppoll(
		&mut self,
		_fds: u64,
		_nfds: u64,
		_timeout: u64,
		_sigmask: u64,
	) -> Result<u64, Error> {
		Ok(0)
	}

	fn handle_rt_sigaction(
		&mut self,
		_signum: u64,
		_action: u64,
		_old_action: u64,
		_sigset_size: u64,
	) -> Result<(), Error> {
		Ok(())
	}

	fn handle_rt_sigprocmask(
		&mut self,
		_signum: u64,
		_action: u64,
		_old_action: u64,
		_sigset_size: u64,
	) -> Result<(), Error> {
		Ok(())
	}
}

fn as_u8_slice<T>(value: &T) -> &[u8] {
	unsafe { core::slice::from_raw_parts(core::ptr::from_ref(value).cast::<u8>(), size_of::<T>()) }
}

/// System call result.
#[derive(Debug)]
pub enum SyscallOutcome {
	/// Continue process execution.
	Continue,
	/// Exit the process with the provided code.
	Exit(u8),
}

const THREAD_ID: u32 = 1;

/// 0, 1, 2 are reserved.
const MAX_RESERVED_FD: u32 = 2;
