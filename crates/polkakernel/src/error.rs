use crate::libc::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Error(pub u64);

impl Error {
	pub const fn code(self) -> u64 {
		errno(self.0)
	}

	pub const fn as_str(self) -> Option<&'static str> {
		Some(match self.0 {
			EACCES => "EACCES",
			EBADF => "EBADF",
			EFAULT => "EFAULT",
			EINVAL => "EINVAL",
			EIO => "EIO",
			ENOENT => "ENOENT",
			ENOSYS => "ENOSYS",
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
