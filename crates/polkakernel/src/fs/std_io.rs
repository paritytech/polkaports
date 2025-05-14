use core::ffi::CStr;
use std::io::{Read, Seek};

use crate::{Error, FileSystem, SeekFrom};

pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
	type Fd = std::fs::File;

	fn open_file(&mut self, path: &CStr) -> Result<Self::Fd, Error> {
		use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
		let path = Path::new(OsStr::from_bytes(path.to_bytes()));
		let file = std::fs::File::open(path)?;
		Ok(file)
	}

	fn seek(&mut self, file: &mut std::fs::File, from: SeekFrom) -> Result<u64, Error> {
		let std_from = match from {
			SeekFrom::Start(x) => std::io::SeekFrom::Start(x),
			SeekFrom::End(x) => std::io::SeekFrom::End(x),
			SeekFrom::Current(x) => std::io::SeekFrom::Current(x),
		};
		Ok(file.seek(std_from)?)
	}

	fn read(&mut self, file: &mut std::fs::File, buf: &mut [u8]) -> Result<u64, Error> {
		Ok(file.read(buf).map(|n| n as u64)?)
	}
}
