use alloc::{borrow::Cow, collections::BTreeMap, ffi::CString, sync::Arc};
use core::ffi::CStr;

// TODO @ivan Support proper file trees.
pub trait FileSystem {
	fn read_file(&mut self, path: &CStr) -> Option<Arc<FileBlob>>;
	fn write_file(&mut self, path: &CStr, data: FileBlob);
}

pub type InMemoryFileSystem = BTreeMap<CString, Arc<FileBlob>>;

impl FileSystem for InMemoryFileSystem {
	fn read_file(&mut self, path: &CStr) -> Option<Arc<FileBlob>> {
		Self::get(self, path).cloned()
	}

	fn write_file(&mut self, path: &CStr, file: FileBlob) {
		self.insert(path.into(), Arc::new(file));
	}
}

pub type FileBlob = Cow<'static, [u8]>;

#[cfg(feature = "std")]
pub struct StdFileSystem;

#[cfg(feature = "std")]
impl FileSystem for StdFileSystem {
	fn read_file(&mut self, path: &CStr) -> Option<Arc<FileBlob>> {
		use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
		let path = Path::new(OsStr::from_bytes(path.to_bytes()));
		let data = match std::fs::read(path) {
			Ok(data) => data,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
			Err(e) => {
				panic!("Failed to read file {path:?}: {e}");
			},
		};
		Some(Arc::new(data.into()))
	}

	fn write_file(&mut self, path: &CStr, data: FileBlob) {
		use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
		let path = Path::new(OsStr::from_bytes(path.to_bytes()));
		let dir = path
			.parent()
			.unwrap_or_else(|| panic!("Failed to get directory of file {path:?}"));
		if let Err(e) = std::fs::create_dir_all(dir) {
			panic!("Failed to create directory {path:?}: {e}");
		}
		if let Err(e) = std::fs::write(path, data.as_ref()) {
			panic!("Failed to write file {path:?}: {e}");
		}
	}
}
