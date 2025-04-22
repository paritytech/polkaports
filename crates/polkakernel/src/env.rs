#[cfg(feature = "std")]
use crate::libc::*;

pub trait Environment {
	fn write_to_stdout(&mut self, data: &[u8]) -> u64;
	fn write_to_stderr(&mut self, data: &[u8]) -> u64;
}

#[cfg(feature = "std")]
pub struct StdEnv;

#[cfg(feature = "std")]
impl Environment for StdEnv {
	fn write_to_stdout(&mut self, data: &[u8]) -> u64 {
		use std::io::Write;
		let stdout = std::io::stdout();
		let mut stdout = stdout.lock();
		match stdout.write(data) {
			Ok(n) => n as u64,
			Err(e) => {
				log::debug!("Error writing to stdout: {e}");
				errno(EIO)
			},
		}
	}

	fn write_to_stderr(&mut self, data: &[u8]) -> u64 {
		use std::io::Write;
		let stderr = std::io::stderr();
		let mut stderr = stderr.lock();
		match stderr.write(data) {
			Ok(n) => n as u64,
			Err(e) => {
				log::debug!("Error writing to stderr: {e}");
				errno(EIO)
			},
		}
	}
}
