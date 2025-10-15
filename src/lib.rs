#![no_std]
#![feature(allocator_api)]

extern crate alloc;

pub mod core;
pub mod drm;
pub mod gsp;
pub mod error;
pub mod memory;
pub mod chipset;

pub use error::{Error, Result};
pub use chipset::{ChipsetSpec, GpuGeneration};
pub const NVIDIA_VENDOR_ID: u16 = 0x10de;
