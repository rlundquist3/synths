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
        fn set_sample_rate(&mut self, sample_rate: u32);
        fn set_freq(&mut self, freq: f32);
    }

    struct EnvelopeParameters {
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    }

    extern "Rust" {
        type Envelope;
        fn new_envelope() -> Box<Envelope>;
        fn set_sample_rate(&mut self, sample_rate: u32);
        fn get_releasing(&self) -> bool;
        fn get_release_complete(&self) -> bool;
        fn set_releasing(&mut self);
        fn set_parameter(&mut self, index: usize, value: f32);
        fn get_parameter_values(&self) -> EnvelopeParameters;
        fn next(&mut self) -> f32;
    }
}
