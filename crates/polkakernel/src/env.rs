use crate::Error;

pub trait Environment {
	fn write_to_stdout(&mut self, data: &[u8]) -> Result<u64, Error>;
	fn write_to_stderr(&mut self, data: &[u8]) -> Result<u64, Error>;
}

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
