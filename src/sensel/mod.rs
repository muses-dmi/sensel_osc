extern crate bitflags;

pub mod bindings;
pub mod scan_mode;
pub mod scan_detail;
mod result;
pub mod device;
pub mod frame;
pub mod contact;

use bindings::*;

pub use result::SenselError;

pub const MAX_DEVICES: usize = bindings::SENSEL_MAX_DEVICES as usize;
