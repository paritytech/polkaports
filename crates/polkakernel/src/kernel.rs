use alloc::{collections::BTreeMap, vec};
use core::{ffi::CStr, mem::size_of};

use crate::{
	libc::*, Environment, Error, FileSystem, IntoSyscallRet, Machine, MachineError, Reg, SeekFrom,
};

use SyscallOutcome::*;

pub struct KernelState<Fd> {
	pub fds: BTreeMap<u64, Fd>,
	/// The next file descriptor after reserved ones.
	pub next_fd: u64,
}

impl<Fd> KernelState<Fd> {
	pub fn new() -> Self {
		Self { fds: BTreeMap::new(), next_fd: 0 }
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
}

impl<C: Machine + Environment + FileSystem> Kernel<C> {
	pub fn handle_syscall(&mut self) -> Result<SyscallOutcome, MachineError> {
		let syscall = self.context.reg(Reg::A0);
		let a1 = self.context.reg(Reg::A1);
		let a2 = self.context.reg(Reg::A2);
		let a3 = self.context.reg(Reg::A3);
		let a4 = self.context.reg(Reg::A4);
		let a5 = self.context.reg(Reg::A5);
		match syscall {
			SYS_READ => {
				let result = self.handle_read(a1, a2, a3).into_ret();
				log::trace!("Syscall read(fd={a1}, address={a2:#x}, length={a3}) = {result}");
				self.context.set_reg(Reg::A0, result);
			},
			SYS_READV => {
				let result = self.handle_readv(a1, a2, a3).into_ret();
				log::trace!("Syscall readv(fd={a1}, iov={a2:#x}, iovcnt={a3}) = {result}");
				self.context.set_reg(Reg::A0, result);
			},
			SYS_WRITE => {
				let result = self.handle_write(a1, a2, a3).into_ret();
				log::trace!("Syscall write(fd={a1}, address={a2:#x}, length={a3}) = {result}");
				self.context.set_reg(Reg::A0, result);
			},
			SYS_WRITEV => {
				let result = self.handle_writev(a1, a2, a3).into_ret();
				log::trace!("Syscall writev(fd={a1}, iov={a2:#x}, iovcnt={a3}) = {result}");
				self.context.set_reg(Reg::A0, result);
			},
			SYS_EXIT => {
				log::info!("Syscall exit(status={a1})");
				return Ok(Exit(a1 as u8));
			},
			SYS_OPENAT => {
				let result = self.handle_openat(a1, a2, a3).into_ret();
				log::trace!("Syscall openat(dirfd={a1}, path={a2:#x}, flags={a3:#o}) = {result}");
				self.context.set_reg(Reg::A0, result);
			},
			SYS_LSEEK => {
				let result = self.handle_lseek(a1, a2 as i64, a3).into_ret();
				log::trace!("Syscall lseek(fd={a1}, offset={a2}, whence={a3}) = {result}");
				self.context.set_reg(Reg::A0, result);
			},
			SYS_CLOSE => {
				let result = self.handle_close(a1).into_ret();
				log::trace!("Syscall close(fd={a1}) = {result}");
				self.context.set_reg(Reg::A0, result);
			},
			SYS_SET_TID_ADDRESS => {
				let result = self.handle_set_tid_address(a1).into_ret();
				log::trace!("Syscall set_tid_address(tid_ptr={a1:#x}) = {result}");
				self.context.set_reg(Reg::A0, result);
			},
			SYS_IOCTL => {
				let result = self.handle_ioctl(a1, a2, a3);
				log::debug!("Syscall ioctl(fd={a1}, op={a2:#x}, {a3}, {a4}, {a5}) = {result:?}");
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

	fn handle_open(&mut self, path: &CStr, flags: u64) -> Result<u64, Error> {
		let file = self.context.open_file(path)?;
		if (flags & (O_WRONLY | O_RDWR)) != 0 {
			return Err(Error(EACCES));
		}
		let fd = RESERVED_FD_COUNT + self.state.next_fd;
		self.state.next_fd += 1;
		self.state.fds.insert(fd, file);
		Ok(fd)
	}

	fn handle_openat(&mut self, dirfd: u64, path: u64, flags: u64) -> Result<u64, Error> {
		if dirfd == AT_FDCWD {
			let path = self.context.read_cstring(path, PATH_MAX)?;
			self.handle_open(&path, flags)
		} else {
			Err(Error(ENOSYS))
		}
	}

	fn handle_close(&mut self, fd: u64) -> Result<u64, Error> {
		self.state.fds.remove(&fd).ok_or(Error(EBADF))?;
		Ok(0)
	}

	fn handle_read(&mut self, fd: u64, address: u64, length: u64) -> Result<u64, Error> {
		let fd = self.state.fds.get_mut(&fd).ok_or(Error(EBADF))?;
		if address.checked_add(length).is_none() || u32::try_from(address + length).is_err() {
			return Err(Error(EFAULT));
		}
		let buf_len = length.try_into().map_err(|_| Error(EFAULT))?;
		let mut buf = vec![0_u8; buf_len];
		let num_bytes_read = self.context.read(fd, &mut buf)?;
		buf.resize(num_bytes_read as usize, 0_u8);
		self.context.write_memory(address, &buf[..])?;
		Ok(num_bytes_read)
	}

	fn handle_readv(&mut self, fd: u64, iov: u64, iovcnt: u64) -> Result<u64, Error> {
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

	fn handle_write(&mut self, fd: u64, address: u64, length: u64) -> Result<u64, Error> {
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

	fn handle_writev(&mut self, fd: u64, iov: u64, iovcnt: u64) -> Result<u64, Error> {
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

	fn handle_lseek(&mut self, fd: u64, offset: i64, whence: u64) -> Result<u64, Error> {
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

	fn handle_ioctl(&mut self, _fd: u64, op: u64, arg0: u64) -> Result<u64, Error> {
		if op == TIOCGWINSZ {
			// NOTE This is a stub to make Musl's `__stdout_write` use line buffering.
			let address = arg0;
			let vt100_size = WinSize { col: 80, row: 25, xpixel: 0, ypixel: 0 };
			let bytes = unsafe {
				core::slice::from_raw_parts(
					core::ptr::from_ref(&vt100_size).cast::<u8>(),
					size_of::<WinSize>(),
				)
			};
			self.context.write_memory(address, bytes)?;
			return Ok(0);
		}
		Err(Error(ENOSYS))
	}
}

#[derive(Debug)]
pub enum SyscallOutcome {
	Continue,
	Exit(u8),
}

const THREAD_ID: u32 = 1;

/// 0, 1, 2 are reserved.
const RESERVED_FD_COUNT: u64 = 3;
