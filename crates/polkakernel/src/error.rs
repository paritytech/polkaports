use crate::libc::*;

/// Syscall error.
///
/// See [errno(3)](https://man7.org/linux/man-pages/man3/errno.3.html).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error(pub u64);

impl Error {
	/// Convert into return code of the syscall.
	pub const fn code(self) -> u64 {
		errno(self.0)
	}

	/// Get error name.
	///
	/// Only some errors are covered by this function.
	pub(crate) const fn as_str(self) -> Option<&'static str> {
		Some(match self.0 {
			EACCES => "EACCES",
			EBADF => "EBADF",
			EFAULT => "EFAULT",
			EINVAL => "EINVAL",
			EIO => "EIO",
			ENOENT => "ENOENT",
			ENOSYS => "ENOSYS",
			EISDIR => "EISDIR",
			ENOTDIR => "ENOTDIR",
			ERANGE => "RANGE",
			_ => return None,
		})
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		match self.as_str() {
			Some(s) => f.write_str(s),
			None => write!(f, "{}", self.0),
		}
	}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		match self.as_str() {
			Some(s) => write!(f, "Error({s})"),
			None => f.debug_tuple("Error").field(&self.0).finish(),
		}
	}
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		use std::io::ErrorKind::*;
		Self(match e.kind() {
			InvalidData => EINVAL,
			InvalidInput => EINVAL,
			NotFound => ENOENT,
			PermissionDenied => EACCES,
			Unsupported => ENOSYS,
			_ => EINVAL,
		})
	}
}

pub(crate) trait IntoSyscallRet {
	fn into_ret(self) -> u64;
}

impl IntoSyscallRet for Result<u64, Error> {
	fn into_ret(self) -> u64 {
		match self {
			Ok(ret) => ret,
			Err(e) => e.code(),
		}
	}
}

impl IntoSyscallRet for Result<u32, Error> {
	fn into_ret(self) -> u64 {
		match self {
			Ok(ret) => u64::from(ret),
			Err(e) => e.code(),
		}
	}
}

impl IntoSyscallRet for Result<(), Error> {
	fn into_ret(self) -> u64 {
		match self {
			Ok(()) => 0,
			Err(e) => e.code(),
		}
	}
}
