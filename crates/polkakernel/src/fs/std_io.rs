//! A local [`FileSystem`].

use core::ffi::CStr;
use std::{
	ffi::OsStr,
	os::{
		fd::RawFd,
		unix::{ffi::OsStrExt, fs::MetadataExt},
	},
	path::Path,
};

use crate::{Error, FileSystem, Metadata, SeekFrom};

/// An implementation of [`FileSystem`] that uses local file system.
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
	type Fd = StdFd;

	fn open(&mut self, path: &CStr, flags: u64) -> Result<Self::Fd, Error> {
		if flags & libc::O_DIRECTORY as u64 != 0 {
			let dir = unsafe { libc::opendir(path.as_ptr()) };
			if dir.is_null() {
				return Err(errno_error());
			}
			Ok(StdFd::Dir(dir))
		} else {
			let raw_fd = check(unsafe { libc::open(path.as_ptr(), flags as i32) })?;
			Ok(StdFd::File(raw_fd))
		}
	}

	fn seek(&mut self, fd: &mut Self::Fd, from: SeekFrom) -> Result<u64, Error> {
		let StdFd::File(ref fd) = fd else {
			return Err(Error(crate::libc::EBADF));
		};
		let (whence, offset): (libc::c_int, libc::off64_t) = match from {
			SeekFrom::Start(x) =>
				(libc::SEEK_SET, x.try_into().map_err(|_| Error(crate::libc::EIO))?),
			SeekFrom::End(x) => (libc::SEEK_END, x),
			SeekFrom::Current(x) => (libc::SEEK_CUR, x),
		};
		let ret = check(unsafe { libc::lseek64(*fd, offset, whence) })?;
		Ok(ret as u64)
	}

	fn read(&mut self, fd: &mut Self::Fd, buf: &mut [u8]) -> Result<usize, Error> {
		let StdFd::File(ref fd) = fd else {
			return Err(Error(crate::libc::EBADF));
		};
		let ret = check(
			unsafe { libc::read(*fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) } as i64,
		)?;
		Ok(ret as usize)
	}

	fn metadata(&mut self, path: &CStr) -> Result<Metadata, Error> {
		let path = Path::new(OsStr::from_bytes(path.to_bytes()));
		let meta = std::fs::metadata(path)?;
		Ok(Metadata {
			size: meta.size(),
			mode: meta.mode(),
			id: meta.ino(),
			block_size: meta.blksize(),
		})
	}

	fn read_dir(&mut self, _fd: &mut Self::Fd, _buf: &mut [u8]) -> Result<usize, Error> {
		// TODO
		Ok(0)
	}
}

#[derive(Debug)]
pub enum StdFd {
	File(RawFd),
	Dir(*mut libc::DIR),
}

impl Drop for StdFd {
	fn drop(&mut self) {
		let result = match self {
			Self::File(fd) => check(unsafe { libc::close(*fd) }),
			Self::Dir(dir) => check(unsafe { libc::closedir(*dir) }),
		};
		if let Err(e) = result {
			log::debug!("Failed to close {self:?}: {e}");
		}
	}
}

fn check<T: Into<i64> + Copy>(ret: T) -> Result<T, Error> {
	if ret.into() < 0_i64 {
		return Err(errno_error());
	}
	Ok(ret)
}

fn errno_error() -> Error {
	let errno = unsafe { libc::__errno_location() };
	Error(unsafe { std::ptr::read::<i32>(errno) } as u64)
}
