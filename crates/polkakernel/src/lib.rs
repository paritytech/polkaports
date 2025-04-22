//! Linux syscall abstraction layer for PolkaVM/CoreVM.

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod env;
mod fs;
mod kernel;
mod libc;
mod machine;
#[cfg(feature = "polkavm-impl")]
pub mod pvm;

pub use self::{env::*, fs::*, kernel::*, machine::*};
