use crate::Error;

/// Execution environment of a user-space program.
pub trait Environment {
	/// Write the provided data to the standard output stream.
	///
	/// Returns the number of bytes written.
	fn write_to_stdout(&mut self, data: &[u8]) -> Result<u64, Error>;

	/// Write the provided data to the standard error stream.
	///
	/// Returns the number of bytes written.
	fn write_to_stderr(&mut self, data: &[u8]) -> Result<u64, Error>;
}

/// An [`Environment`] that uses Rust standard library for I/O.
#[cfg(feature = "std")]
pub struct StdEnv;

#[cfg(feature = "std")]
impl Environment for StdEnv {
	fn write_to_stdout(&mut self, data: &[u8]) -> Result<u64, Error> {
		use std::io::Write;
		let stdout = std::io::stdout();
		let mut stdout = stdout.lock();
		Ok(stdout.write(data).map(|n| n as u64)?)
	}

	fn write_to_stderr(&mut self, data: &[u8]) -> Result<u64, Error> {
		use std::io::Write;
		let stderr = std::io::stderr();
		let mut stderr = stderr.lock();
		Ok(stderr.write(data).map(|n| n as u64)?)
	}
}
