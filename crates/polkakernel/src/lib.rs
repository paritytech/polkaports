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

pub use self::{env::*, fs::*, kernel::*, machine::*};
