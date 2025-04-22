use alloc::{collections::BTreeMap, sync::Arc};
use core::ffi::CStr;

use crate::{libc::*, Environment, FileBlob, FileSystem, Machine, MachineError, Reg};

use SyscallOutcome::*;

pub struct Kernel<M: Machine, E: Environment, F: FileSystem> {
	machine: M,
	env: E,
	fs: F,
	fds: BTreeMap<u64, Fd>,
	next_fd: u64,
}

impl<M: Machine, E: Environment, F: FileSystem> Kernel<M, E, F> {
	pub fn new(machine: M, env: E, fs: F) -> Self {
		Self { machine, env, fs, fds: BTreeMap::new(), next_fd: 3 }
	}

	pub fn machine(&self) -> &M {
		&self.machine
	}

	pub fn machine_mut(&mut self) -> &mut M {
		&mut self.machine
	}

	pub fn env(&self) -> &E {
		&self.env
	}

	pub fn env_mut(&mut self) -> &mut E {
		&mut self.env
	}

	pub fn fs(&self) -> &F {
		&self.fs
	}

	pub fn fs_mut(&mut self) -> &mut F {
		&mut self.fs
	}

	pub fn into_inner(self) -> (M, E) {
		(self.machine, self.env)
	}

	pub fn init<'argv, 'envp, I1, I2>(
		&mut self,
		default_sp: u64,
		default_ra: u64,
		start: u64,
		argv: I1,
		envp: I2,
	) -> Result<(), MachineError>
	where
		I1: IntoIterator<Item = &'argv CStr>,
		<I1 as IntoIterator>::IntoIter: ExactSizeIterator,
		I2: IntoIterator<Item = &'envp CStr>,
		<I2 as IntoIterator>::IntoIter: ExactSizeIterator,
	{
		let argv = argv.into_iter();
		let argc = argv.len() as u64;
		let envp = envp.into_iter();
		let envp_len = envp.len() as u64;
		let auxv: &[(u64, u64)] = &[(AT_PAGESZ, 4096)];
		let auxv_len = auxv.len() as u64;

		let mut sp = default_sp;

		sp -= (1 + argc + 1 + envp_len + 1 + (auxv_len + 1) * 2) * 8;
		let address_init = sp;

		let mut p = sp;
		self.machine.write_u64(p, argc)?;
		p += 8;

		for arg in argv {
			let bytes = arg.to_bytes();
			sp -= bytes.len() as u64 + 1;
			self.machine.write_memory(sp, bytes)?;
			self.machine.write_u64(p, sp)?;
			p += 8;
		}
		p += 8; // Null pointer.

		for arg in envp {
			let bytes = arg.to_bytes();
			sp -= bytes.len() as u64 + 1;
			self.machine.write_memory(sp, bytes)?;
			self.machine.write_u64(p, sp)?;
			p += 8;
		}
		p += 8; // Null pointer.

		for &(key, value) in auxv {
			self.machine.write_u64(p, key)?;
			p += 8;
			self.machine.write_u64(p, value)?;
			p += 8;
		}

		self.machine.set_reg(Reg::SP, sp);
		self.machine.set_reg(Reg::A0, address_init);
		self.machine.set_reg(Reg::RA, default_ra);
		self.machine.set_next_program_counter(start);
		Ok(())
	}

	pub fn handle_syscall(&mut self) -> Result<SyscallOutcome, MachineError> {
		let syscall = self.machine.reg(Reg::A0);
		let a1 = self.machine.reg(Reg::A1);
		let a2 = self.machine.reg(Reg::A2);
		let a3 = self.machine.reg(Reg::A3);
		let a4 = self.machine.reg(Reg::A4);
		let a5 = self.machine.reg(Reg::A5);
		let pc = self.machine.program_counter();
		log::trace!(
			"Syscall at pc={pc}: {syscall:>3}, \
            args = [0x{a1:>016x}, 0x{a2:>016x}, 0x{a3:>016x}, 0x{a4:>016x}, 0x{a5:>016x}]"
		);
		match syscall {
			SYS_READ => {
				let result = self.handle_read(a1, a2, a3);
				self.machine.set_reg(Reg::A0, result);
			},
			SYS_READV => {
				let result = self.handle_readv(a1, a2, a3);
				self.machine.set_reg(Reg::A0, result);
			},
			SYS_WRITE => {
				let result = self.handle_write(a1, a2, a3);
				self.machine.set_reg(Reg::A0, result);
			},
			SYS_WRITEV => {
				let result = self.handle_writev(a1, a2, a3);
				self.machine.set_reg(Reg::A0, result);
			},
			SYS_EXIT => {
				log::info!("Exit called: status={}", a1);
				return Ok(Exit(a1 as u8));
			},
			SYS_OPENAT => {
				let result = self.handle_openat(a1, a2, a3);
				self.machine.set_reg(Reg::A0, result);
			},
			SYS_LSEEK => {
				let result = self.handle_lseek(a1, a2 as i64, a3);
				self.machine.set_reg(Reg::A0, result);
			},
			SYS_CLOSE => {
				let result = self.handle_close(a1);
				self.machine.set_reg(Reg::A0, result);
			},
			_ => {
				log::debug!(
					"Unimplemented syscall at pc={pc}: {syscall:>3}, \
                    args = [0x{a1:>016x}, 0x{a2:>016x}, 0x{a3:>016x}, 0x{a4:>016x}, 0x{a5:>016x}]"
				);
				self.machine.set_reg(Reg::A0, errno(ENOSYS));
			},
		}
		Ok(Continue)
	}

	fn handle_open(&mut self, path: &CStr, flags: u64) -> u64 {
		log::debug!("Open: path={:?}, flags={:#o}", path, flags);

		if let Some(file) = self.fs.read_file(path) {
			if (flags & (O_WRONLY | O_RDWR)) != 0 {
				log::trace!("  -> EACCES");
				return errno(EACCES);
			}

			let fd = self.next_fd;
			log::trace!("  -> fd={fd}");

			self.next_fd += 1;
			self.fds.insert(fd, Fd { file, position: 0 });

			return fd;
		}

		log::trace!("  -> ENOENT");
		errno(ENOENT)
	}

	fn handle_openat(&mut self, dirfd: u64, path: u64, flags: u64) -> u64 {
		if dirfd == AT_FDCWD {
			let path = match self.machine.read_cstring(path, PATH_MAX) {
				Ok(path) => path,
				Err(MachineError::BadAddress) => return errno(EFAULT),
			};
			self.handle_open(&path, flags)
		} else {
			errno(ENOSYS)
		}
	}

	fn handle_close(&mut self, fd: u64) -> u64 {
		log::debug!("Close: fd = {fd}");
		let Some(_fd) = self.fds.remove(&fd) else {
			log::trace!("  -> EBADF");
			return errno(EBADF);
		};

		0
	}

	fn handle_read(&mut self, fd: u64, address: u64, length: u64) -> u64 {
		log::trace!("Read: fd={fd}, address=0x{address:x}, length={length}");

		let Some(fd) = self.fds.get_mut(&fd) else {
			log::trace!("  -> EBADF");
			return errno(EBADF);
		};

		if address.checked_add(length).is_none() || u32::try_from(address + length).is_err() {
			log::trace!("  -> EFAULT");
			return errno(EFAULT);
		}

		let end = core::cmp::min(fd.position.wrapping_add(length), fd.file.len() as u64);
		if fd.position >= end || fd.position >= fd.file.len() as u64 {
			log::trace!("  -> offset={}, length=0", fd.position);
			return 0;
		}

		let blob = &fd.file[fd.position as usize..end as usize];
		match self.machine.write_memory(address, blob) {
			Ok(()) => {},
			Err(MachineError::BadAddress) => {
				log::trace!("  -> EFAULT");
				return errno(EFAULT);
			},
		}

		let length_out = blob.len() as u64;
		log::trace!(
			"  -> offset={}, length={}, new offset={}",
			fd.position,
			length_out,
			fd.position + length_out
		);

		fd.position += length_out;
		length_out
	}

	fn handle_readv(&mut self, fd: u64, iov: u64, iovcnt: u64) -> u64 {
		if iovcnt == 0 || iovcnt > IOV_MAX {
			return errno(EINVAL);
		}

		let mut total_length = 0;
		for n in 0..iovcnt {
			let address = match self.machine.read_u64(iov.wrapping_add(n * 16)) {
				Ok(address) => address,
				Err(MachineError::BadAddress) => return errno(EFAULT),
			};
			let length = match self.machine.read_u64(iov.wrapping_add(n * 16).wrapping_add(8)) {
				Ok(length) => length,
				Err(MachineError::BadAddress) => return errno(EFAULT),
			};
			let errcode = self.handle_read(fd, address, length);
			if (errcode as i64) < 0 {
				return errcode;
			}

			total_length += length;
		}

		total_length
	}

	fn handle_write(&mut self, fd: u64, address: u64, length: u64) -> u64 {
		if fd != FILENO_STDOUT && fd != FILENO_STDERR {
			return errno(EBADF);
		}

		if address.checked_add(length).is_none() || u32::try_from(address + length).is_err() {
			return errno(EFAULT);
		}

		let data = match self.machine.read_memory(address, length) {
			Ok(data) => data,
			Err(MachineError::BadAddress) => return errno(EFAULT),
		};

		if fd == FILENO_STDOUT {
			self.env.write_to_stdout(&data[..])
		} else {
			self.env.write_to_stderr(&data[..])
		}
	}

	fn handle_writev(&mut self, fd: u64, iov: u64, iovcnt: u64) -> u64 {
		if iovcnt == 0 || iovcnt > IOV_MAX {
			return errno(EINVAL);
		}

		let mut total_length = 0;
		for n in 0..iovcnt {
			let address = match self.machine.read_u64(iov.wrapping_add(n * 16)) {
				Ok(address) => address,
				Err(MachineError::BadAddress) => return errno(EFAULT),
			};
			let length = match self.machine.read_u64(iov.wrapping_add(n * 16).wrapping_add(8)) {
				Ok(length) => length,
				Err(MachineError::BadAddress) => return errno(EFAULT),
			};
			let errcode = self.handle_write(fd, address, length);
			if (errcode as i64) < 0 {
				return errcode;
			}

			total_length += length;
		}

		total_length
	}

	fn handle_lseek(&mut self, fd: u64, offset: i64, whence: u64) -> u64 {
		log::trace!("Seek: fd={fd}, offset={offset}, whence={whence}");

		let Some(fd) = self.fds.get_mut(&fd) else {
			log::trace!("  -> BADF");
			return errno(EBADF);
		};

		match whence {
			SEEK_SET => {
				fd.position = offset as u64;
			},
			SEEK_CUR => {
				fd.position = core::cmp::min(
					(fd.position as i64).wrapping_add(offset) as u64,
					fd.file.len() as u64,
				);
			},
			SEEK_END => {
				fd.position = core::cmp::min(
					(fd.file.len() as i64).wrapping_add(offset) as u64,
					fd.file.len() as u64,
				);
			},
			_ => {
				log::trace!("  -> EINVAL");
				return errno(EINVAL);
			},
		}

		log::trace!("  -> offset={}", fd.position);
		fd.position
	}
}

#[derive(Debug)]
pub enum SyscallOutcome {
	Continue,
	Exit(u8),
}

struct Fd {
	file: Arc<FileBlob>,
	position: u64,
}
