use alloc::{borrow::Cow, collections::BTreeMap, ffi::CString, sync::Arc};
use core::ffi::CStr;

// TODO @ivan Support proper file trees.
pub trait FileSystem {
	fn read_file(&self, path: &CStr) -> Option<Arc<File>>;
	fn write_file(&mut self, path: &CStr, data: FileBlob);
}

pub type InMemoryFileSystem = BTreeMap<CString, Arc<File>>;

impl FileSystem for InMemoryFileSystem {
	fn read_file(&self, path: &CStr) -> Option<Arc<File>> {
		Self::get(self, path).cloned()
	}

	fn write_file(&mut self, path: &CStr, data: FileBlob) {
		let file = File { blob: data };
		self.insert(path.into(), Arc::new(file));
	}
}

// TODO flatten?
pub struct File {
	pub(crate) blob: FileBlob,
}

impl File {
	pub fn from_blob(blob: FileBlob) -> Self {
		Self { blob }
	}
}

pub type FileBlob = Cow<'static, [u8]>;

#[cfg(feature = "std")]
pub struct StdFileSystem;

#[cfg(feature = "std")]
impl FileSystem for StdFileSystem {
	fn read_file(&self, path: &CStr) -> Option<Arc<File>> {
		use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
		let path = Path::new(OsStr::from_bytes(path.to_bytes()));
		let data = match std::fs::read(path) {
			Ok(data) => data,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
			Err(e) => {
				panic!("Failed to read file {path:?}: {e}");
			},
		};
		let file = File { blob: data.into() };
		Some(Arc::new(file))
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
