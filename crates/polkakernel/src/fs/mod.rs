#[cfg(feature = "std")]
mod std_io;

#[cfg(feature = "std")]
pub use self::std_io::*;

mod in_memory;

pub use self::in_memory::*;

use alloc::{ffi::CString, vec::Vec};
use core::ffi::CStr;

use crate::Error;

pub trait FileSystem {
	type Fd: Sized;

	fn open(&mut self, path: &CStr, flags: u64) -> Result<Self::Fd, Error>;
	fn seek(&mut self, fd: &mut Self::Fd, from: SeekFrom) -> Result<u64, Error>;
	fn read(&mut self, fd: &mut Self::Fd, buf: &mut [u8]) -> Result<usize, Error>;
	fn read_dir(&mut self, fd: &mut Self::Fd, buf: &mut [u8]) -> Result<usize, Error>;
	fn metadata(&mut self, path: &CStr) -> Result<Metadata, Error>;
}

#[derive(Debug, Clone)]
pub struct Metadata {
	pub id: u64,
	pub size: u64,
	pub mode: u32,
	pub block_size: u64,
	pub num_blocks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
	Start(u64),
	End(i64),
	Current(i64),
}

pub const fn dir_entry_len(name_len: usize) -> usize {
	let real_len = 8 + 8 + 2 + 1 + name_len;
	// Align to 8-byte boundary.
	real_len.next_multiple_of(8)
}

pub fn write_dir_entry(id: u64, name: &CStr, buf: &mut [u8]) -> usize {
	let name_bytes = name.to_bytes_with_nul();
	let name_len = name_bytes.len();
	let Ok(entry_len) = u16::try_from(dir_entry_len(name_len)) else {
		return 0;
	};
	let off = 0_u64;
	let d_type = 0_u8;
	buf[0..8].copy_from_slice(id.to_le_bytes().as_slice());
	buf[8..16].copy_from_slice(off.to_le_bytes().as_slice());
	buf[16..18].copy_from_slice(entry_len.to_le_bytes().as_slice());
	buf[18] = d_type;
	let n = 19 + name_len;
	buf[19..n].copy_from_slice(name_bytes);
	assert!(n <= entry_len as usize);
	entry_len.into()
}

pub fn normalize_path(path: &CStr) -> CString {
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
