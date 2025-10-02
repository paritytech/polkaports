#[cfg(feature = "std")]
pub mod std_io;

pub mod in_memory;

use alloc::{ffi::CString, vec::Vec};
use core::ffi::CStr;

use crate::Error;

/// File system of a user-space program.
pub trait FileSystem {
	type Fd: Sized;

	/// Open file under the provided path.
	///
	/// See [open(2)](https://man7.org/linux/man-pages/man2/open.2.html).
	fn open(&mut self, path: &CStr, flags: u64) -> Result<Self::Fd, Error>;

	/// Set the current read/write offset of the opened file.
	///
	/// See [lseek(2)](https://man7.org/linux/man-pages/man2/lseek.2.html).
	fn seek(&mut self, fd: &mut Self::Fd, from: SeekFrom) -> Result<u64, Error>;

	/// Read data from the opened file to the provided buffer.
	///
	/// Returns the number of bytes read.
	///
	/// See [read(2)](https://man7.org/linux/man-pages/man2/read.2.html).
	fn read(&mut self, fd: &mut Self::Fd, buf: &mut [u8]) -> Result<usize, Error>;

	/// Read directory contents into the provided buffer.
	///
	/// The implementation is expected to call [`write_dir_entry`] with `buf` as the last argument
	/// for each entry in the directory.
	///
	/// See [readdir(2)](https://man7.org/linux/man-pages/man2/readdir.2.html).
	fn read_dir(&mut self, fd: &mut Self::Fd, buf: &mut [u8]) -> Result<usize, Error>;

	/// Read file metadata from the provided path.
	fn metadata(&mut self, path: &CStr) -> Result<Metadata, Error>;
}

/// File system node metadata.
///
/// See [stat(3type)](https://man7.org/linux/man-pages/man3/stat.3type.html).
#[derive(Debug, Clone)]
pub struct Metadata {
	pub id: u64,
	pub size: u64,
	pub mode: u32,
	pub block_size: u64,
}

/// File read/write position anchor.
///
/// See [lseek(2)](https://man7.org/linux/man-pages/man2/lseek.2.html).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
	/// Offset from the start of the file.
	Start(u64),
	/// Offset from the end of the file.
	End(i64),
	/// Offset from the current read/write position.
	Current(i64),
}

/// Get directory entry length for the provided file name length.
///
/// See [getdents(2)](https://man7.org/linux/man-pages/man2/getdents.2.html).
pub const fn dir_entry_len(name_len: usize) -> usize {
	let real_len = 8 + 8 + 2 + 1 + name_len;
	// Align to 8-byte boundary.
	real_len.next_multiple_of(8)
}

/// Write directory entry for the specified file name to the provided buffer.
///
/// Meant to be used in the implementation of [`FileSystem::read_dir`].
///
/// See [getdents(2)](https://man7.org/linux/man-pages/man2/getdents.2.html).
pub fn write_dir_entry(id: u64, name: &CStr, buf: &mut [u8]) -> Result<usize, WriteDirEntryErr> {
	use WriteDirEntryErr::*;
	let name_bytes = name.to_bytes_with_nul();
	let name_len = name_bytes.len();
	let entry_len = dir_entry_len(name_len);
	if entry_len > buf.len() {
		return Err(BufferTooSmall);
	}
	let entry_len_u16: u16 = entry_len.try_into().map_err(|_| NameTooLong)?;
	let off = 0_u64;
	let d_type = 0_u8;
	buf[0..8].copy_from_slice(id.to_le_bytes().as_slice());
	buf[8..16].copy_from_slice(off.to_le_bytes().as_slice());
	buf[16..18].copy_from_slice(entry_len_u16.to_le_bytes().as_slice());
	buf[18] = d_type;
	let n = 19 + name_len;
	buf[19..n].copy_from_slice(name_bytes);
	Ok(entry_len)
}

#[derive(Debug)]
pub enum WriteDirEntryErr {
	NameTooLong,
	BufferTooSmall,
}

pub(crate) fn normalize_path(path: &CStr) -> CString {
	let path = path.to_bytes();
	let mut components = Vec::new();
	for comp in path.split(|byte| *byte == b'/') {
		match comp {
			b"" | b"." => {
				// Nom-nom-nom.
			},
			b".." => {
				components.pop();
			},
			comp => components.push(comp),
		}
	}
	let mut normal = Vec::new();
	if path.first() == Some(&b'/') {
		normal.push(b'/');
	}
	let mut iter = components.into_iter();
	if let Some(comp) = iter.next() {
		normal.extend_from_slice(comp);
	}
	for comp in iter {
		normal.push(b'/');
		normal.extend_from_slice(comp);
	}
	normal.push(0_u8);
	// SAFETY: We add NUL byte ourselves.
	unsafe { CString::from_vec_with_nul_unchecked(normal) }
}
