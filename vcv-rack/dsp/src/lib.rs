mod envelope;
mod oscillator;

use envelope::{Envelope, new_envelope};
use oscillator::{Oscillator, new_oscillator};

#[cxx::bridge]
mod ffi {
    struct SamplesByWaveform {
        sine: f32,
        sawtooth: f32,
        square: f32,
        triangle: f32,
    }

    extern "Rust" {
        type Oscillator;
        fn new_oscillator() -> Box<Oscillator>;
        fn next_sample_by_waveform(&mut self) -> SamplesByWaveform;
        // fn set_sample_rate(&mut self, freq: f32);
        fn set_freq(&mut self, freq: f32);
    }

    extern "Rust" {
        type Envelope;
        fn new_envelope() -> Box<Envelope>;
        fn get_releasing(&self) -> bool;
        fn get_release_complete(&self) -> bool;
        fn set_releasing(&mut self);
        fn decrement_parameter(&mut self, index: usize);
        fn increment_parameter(&mut self, index: usize);
        fn next(&mut self) -> f32;
    }
}
