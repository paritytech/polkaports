use alloc::{ffi::CString, vec::Vec};

use MachineError::*;

pub trait Machine {
	fn program_counter(&self) -> u64;
	fn set_next_program_counter(&mut self, pc: u64);

	fn reg(&self, name: Reg) -> u64;
	fn set_reg(&mut self, name: Reg, value: u64);

	fn read_u64(&self, address: u64) -> Result<u64, MachineError>;
	fn read_u32(&self, address: u64) -> Result<u32, MachineError>;
	fn read_u16(&self, address: u64) -> Result<u16, MachineError>;
	fn read_u8(&self, address: u64) -> Result<u8, MachineError>;

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

	fn read_memory_into(&self, address: u64, buffer: &mut [u8]) -> Result<(), MachineError>;

	fn read_memory(&self, address: u64, length: u64) -> Result<Vec<u8>, MachineError> {
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
