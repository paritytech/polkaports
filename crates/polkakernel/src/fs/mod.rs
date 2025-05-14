#[cfg(feature = "std")]
mod std_io;

#[cfg(feature = "std")]
pub use self::std_io::*;

mod in_memory;

pub use self::in_memory::*;

use alloc::{ffi::CString, vec::Vec};
use core::ffi::CStr;

use crate::Error;

// TODO @ivan Support proper file trees.
pub trait FileSystem {
	type Fd: Sized;

	fn open_file(&mut self, path: &CStr) -> Result<Self::Fd, Error>;
	fn seek(&mut self, fd: &mut Self::Fd, from: SeekFrom) -> Result<u64, Error>;
	fn read(&mut self, fd: &mut Self::Fd, buf: &mut [u8]) -> Result<u64, Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
	Start(u64),
	End(i64),
	Current(i64),
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
