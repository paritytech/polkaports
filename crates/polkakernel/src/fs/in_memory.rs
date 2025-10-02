//! In-memory [`FileSystem`].

use alloc::{borrow::Cow, collections::BTreeMap, ffi::CString, sync::Arc};
use core::ffi::CStr;

use crate::{libc::*, normalize_path, Error, FileSystem, Metadata, SeekFrom};

/// In-memory file contents.
pub type FileBlob = Cow<'static, [u8]>;

/// An implementation of [`FileSystem`] that uses memory to store files.
pub type InMemoryFileSystem = BTreeMap<CString, Arc<FileBlob>>;

/// In-memory-specific file system error.
#[derive(Debug)]
pub enum InMemoryError {
	NotFound,
}

impl FileSystem for InMemoryFileSystem {
	type Fd = InMemoryFd;

	fn open(&mut self, path: &CStr, _flags: u64) -> Result<Self::Fd, Error> {
		let path = normalize_path(path);
		Self::get(self, &path)
			.cloned()
			.map(|blob| InMemoryFd { position: 0, blob })
			.ok_or(Error(ENOENT))
	}

	fn seek(&mut self, fd: &mut InMemoryFd, from: SeekFrom) -> Result<u64, Error> {
		match from {
			SeekFrom::Start(offset) => fd.position = offset,
			SeekFrom::Current(offset) =>
				fd.position = ((fd.position as i64).wrapping_add(offset) as u64).min(fd.size()),
			SeekFrom::End(offset) => {
				let size = fd.size();
				fd.position = ((size as i64).wrapping_add(offset) as u64).min(size);
			},
		}
		Ok(fd.position)
	}

	fn read(&mut self, fd: &mut InMemoryFd, buf: &mut [u8]) -> Result<usize, Error> {
		let size = fd.size();
		let end = core::cmp::min(fd.position.wrapping_add(buf.len() as u64), size);
		if fd.position >= end || fd.position >= size {
			log::trace!("  -> offset={}, length=0", fd.position);
			return Ok(0);
		}
		let slice = &fd.blob[fd.position as usize..end as usize];
		let num_bytes_read = slice.len();
		buf.copy_from_slice(slice);
		log::trace!(
			"  -> offset={}, length={}, new offset={}",
			fd.position,
			num_bytes_read,
			fd.position + num_bytes_read as u64
		);
		fd.position += num_bytes_read as u64;
		Ok(num_bytes_read)
	}

	fn metadata(&mut self, path: &CStr) -> Result<Metadata, Error> {
		let path = normalize_path(path);
		Self::get(self, &path)
			.map(|blob| Metadata {
				id: blob.as_ptr() as u64,
				mode: 0o100644,
				size: blob.len() as u64,
				block_size: blob.len() as u64,
			})
			.ok_or(Error(ENOENT))
	}

	fn read_dir(&mut self, _fd: &mut Self::Fd, _buf: &mut [u8]) -> Result<usize, Error> {
		Err(Error(ENOSYS))
	}
}

pub struct InMemoryFd {
	pub position: u64,
	pub blob: Arc<FileBlob>,
}

impl InMemoryFd {
	fn size(&self) -> u64 {
		self.blob.len() as u64
	}
}
