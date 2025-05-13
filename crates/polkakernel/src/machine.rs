use alloc::{ffi::CString, vec::Vec};
use core::ffi::CStr;

use crate::libc::*;

use MachineError::*;

pub trait Machine {
	fn init<'argv, 'envp, I1, I2>(
		&mut self,
		default_sp: u64,
		default_ra: u64,
		argv: I1,
		envp: I2,
	) -> Result<(), MachineError>
	where
		I1: IntoIterator<Item = &'argv CStr>,
		<I1 as IntoIterator>::IntoIter: ExactSizeIterator,
		I2: IntoIterator<Item = &'envp CStr>,
		<I2 as IntoIterator>::IntoIter: ExactSizeIterator,
	{
		let argv = argv.into_iter();
		let argc = argv.len() as u64;
		let envp = envp.into_iter();
		let envp_len = envp.len() as u64;
		let auxv: &[(u64, u64)] = &[(AT_PAGESZ, 4096)];
		let auxv_len = auxv.len() as u64;

		let mut sp = default_sp;

		sp -= (1 + argc + 1 + envp_len + 1 + (auxv_len + 1) * 2) * 8;
		let address_init = sp;
		self.touch_memory(address_init, default_sp)?;

		let mut p = sp;
		self.write_u64(p, argc)?;
		p += 8;

		for arg in argv {
			let bytes = arg.to_bytes();
			sp -= bytes.len() as u64 + 1;
			self.write_memory(sp, bytes)?;
			self.write_u64(p, sp)?;
			p += 8;
		}
		p += 8; // Null pointer.

		for arg in envp {
			let bytes = arg.to_bytes();
			sp -= bytes.len() as u64 + 1;
			self.write_memory(sp, bytes)?;
			self.write_u64(p, sp)?;
			p += 8;
		}
		p += 8; // Null pointer.

		for &(key, value) in auxv {
			self.write_u64(p, key)?;
			p += 8;
			self.write_u64(p, value)?;
			p += 8;
		}

		self.set_reg(Reg::SP, sp);
		self.set_reg(Reg::A0, address_init);
		self.set_reg(Reg::RA, default_ra);
		Ok(())
	}

	fn reg(&self, name: Reg) -> u64;
	fn set_reg(&mut self, name: Reg, value: u64);

	fn read_u64(&mut self, address: u64) -> Result<u64, MachineError>;
	fn read_u32(&mut self, address: u64) -> Result<u32, MachineError>;
	fn read_u16(&mut self, address: u64) -> Result<u16, MachineError>;
	fn read_u8(&mut self, address: u64) -> Result<u8, MachineError>;

	/// Read C-string from the provided address.
	///
	/// `max_len` is the maximum number of bytes in a string (including the NUL byte).
	///
	/// The default implementation reads string byte-by-byte. Implementers should consider providing
	/// a more performant version of this method.
	fn read_cstring(&mut self, address: u64, max_len: u64) -> Result<CString, MachineError> {
		let mut buffer = Vec::new();
		for offset in address..address.saturating_add(max_len) {
			let byte = self.read_u8(offset)?;
			if byte == 0 {
				buffer.push(0);
				// SAFETY: We check for the NUL byte ourselves.
				let c_string = unsafe { CString::from_vec_with_nul_unchecked(buffer) };
				return Ok(c_string);
			}
			buffer.push(byte)
		}
		// Haven't found NUL byte.
		Err(BadAddress)
	}

	fn read_memory_into(&mut self, address: u64, buffer: &mut [u8]) -> Result<(), MachineError>;

	fn read_memory(&mut self, address: u64, length: u64) -> Result<Vec<u8>, MachineError> {
		let Ok(len) = length.try_into() else {
			return Err(BadAddress);
		};
		let mut buf = Vec::with_capacity(len);
		self.read_memory_into(
			address,
			// SAFETY: `ptr` points to an allocated memory region.
			unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr(), len) },
		)?;
		// SAFETY: `read_memory_into` initializes all pre-allocated bytes in the vector.
		unsafe { buf.set_len(len) };
		Ok(buf)
	}

	fn write_u64(&mut self, address: u64, value: u64) -> Result<(), MachineError>;
	fn write_memory(&mut self, address: u64, slice: &[u8]) -> Result<(), MachineError>;

	fn touch_memory(&mut self, start_address: u64, end_address: u64) -> Result<(), MachineError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineError {
	BadAddress,
}

impl core::fmt::Display for MachineError {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		core::fmt::Debug::fmt(self, f)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for MachineError {}

/// Available registers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
pub enum Reg {
	RA = 0,
	SP = 1,
	T0 = 2,
	T1 = 3,
	T2 = 4,
	S0 = 5,
	S1 = 6,
	A0 = 7,
	A1 = 8,
	A2 = 9,
	A3 = 10,
	A4 = 11,
	A5 = 12,
}
