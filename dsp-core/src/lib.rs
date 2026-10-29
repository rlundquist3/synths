#![no_std]

pub mod amp_envelope;
pub mod buffer_pool;
pub mod chain;
pub mod effects;
pub mod engines;
pub mod midi;
pub mod oscillator;
pub mod parameter;
pub mod utils;
pub mod voices;

extern crate alloc;

// TODO: might want to make this a member of the various structs that use it, with setters
pub static SAMPLE_RATE: u32 = 48_000;
