#![no_std]

pub mod adsr_envelope;
pub mod buffer_pool;
pub mod chain;
pub mod controls;
pub mod effects;
pub mod engines;
pub mod midi;
pub mod oscillator;
pub mod parameter;
pub mod utils;
pub mod voices;

extern crate alloc;

pub static DEFAULT_SAMPLE_RATE: u32 = 48_000;
