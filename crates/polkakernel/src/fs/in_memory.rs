use alloc::{borrow::Cow, collections::BTreeMap, ffi::CString, sync::Arc};
use core::ffi::CStr;

use crate::{libc::*, normalize_path, Error, FileSystem, SeekFrom};

pub type FileBlob = Cow<'static, [u8]>;
pub type InMemoryFileSystem = BTreeMap<CString, Arc<FileBlob>>;

#[derive(Debug)]
pub enum InMemoryError {
	NotFound,
}

impl FileSystem for InMemoryFileSystem {
	type Fd = InMemoryFd;

	fn open_file(&mut self, path: &CStr) -> Result<Self::Fd, Error> {
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

	fn read(&mut self, fd: &mut InMemoryFd, buf: &mut [u8]) -> Result<u64, Error> {
		let size = fd.size();
		let end = core::cmp::min(fd.position.wrapping_add(buf.len() as u64), size);
		if fd.position >= end || fd.position >= size {
			log::trace!("  -> offset={}, length=0", fd.position);
			return Ok(0);
		}
		let slice = &fd.blob[fd.position as usize..end as usize];
		let num_bytes_read = slice.len() as u64;
		buf.copy_from_slice(slice);
		log::trace!(
			"  -> offset={}, length={}, new offset={}",
			fd.position,
			num_bytes_read,
			fd.position + num_bytes_read
		);
		fd.position += num_bytes_read;
		Ok(num_bytes_read)
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
