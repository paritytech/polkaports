use polkavm::{ProgramCounter, RawInstance};

use crate::{Machine, MachineError, MachineError::*, Reg};

impl From<Reg> for polkavm::Reg {
	fn from(other: Reg) -> Self {
		unsafe { core::mem::transmute(other) }
	}
}

impl Machine for RawInstance {
	fn program_counter(&self) -> u64 {
		RawInstance::program_counter(self)
			.unwrap_or_else(|| panic!("Failed to get program counter"))
			.0
			.into()
	}

	fn set_next_program_counter(&mut self, pc: u64) {
		let pc: u32 = pc.try_into().unwrap_or_else(|_| panic!("Failed to set program counter"));
		RawInstance::set_next_program_counter(self, ProgramCounter(pc));
	}

	fn reg(&self, name: Reg) -> u64 {
		RawInstance::reg(self, name.into())
	}

	fn set_reg(&mut self, name: Reg, value: u64) {
		RawInstance::set_reg(self, name.into(), value);
	}

	fn read_u64(&self, address: u64) -> Result<u64, MachineError> {
		let address = address.try_into().map_err(|_| BadAddress)?;
		RawInstance::read_u64(self, address).map_err(|_| BadAddress)
	}

	fn read_u32(&self, address: u64) -> Result<u32, MachineError> {
		let address = address.try_into().map_err(|_| BadAddress)?;
		RawInstance::read_u32(self, address).map_err(|_| BadAddress)
	}

	fn read_u16(&self, address: u64) -> Result<u16, MachineError> {
		let address = address.try_into().map_err(|_| BadAddress)?;
		RawInstance::read_u16(self, address).map_err(|_| BadAddress)
	}

	fn read_u8(&self, address: u64) -> Result<u8, MachineError> {
		let address = address.try_into().map_err(|_| BadAddress)?;
		RawInstance::read_u8(self, address).map_err(|_| BadAddress)
	}

	fn read_memory_into(&self, address: u64, buffer: &mut [u8]) -> Result<(), MachineError> {
		let address = address.try_into().map_err(|_| BadAddress)?;
		RawInstance::read_memory_into(self, address, buffer).map_err(|_| BadAddress)?;
		Ok(())
	}

	fn write_u64(&mut self, address: u64, value: u64) -> Result<(), MachineError> {
		let address = address.try_into().map_err(|_| BadAddress)?;
		RawInstance::write_u64(self, address, value).map_err(|_| BadAddress)
	}

	fn write_memory(&mut self, address: u64, slice: &[u8]) -> Result<(), MachineError> {
		let address = address.try_into().map_err(|_| BadAddress)?;
		RawInstance::write_memory(self, address, slice).map_err(|_| BadAddress)
	}
}
