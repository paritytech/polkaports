//! Linux syscall abstraction layer for PolkaVM/CoreVM.

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod env;
mod error;
mod fs;
mod kernel;
pub mod libc;
mod machine;

pub use self::{env::*, error::*, fs::*, kernel::*, machine::*};
