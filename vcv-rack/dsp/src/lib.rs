use cxx::CxxVector;
use ffi::*;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type Oscillator;
        fn new_oscillator() -> Box<Oscillator>;
        fn next_sample(&mut self) -> f32;
        // fn set_sample_rate(&mut self, freq: f32);
        fn set_freq(&mut self, freq: f32);
    }
}

pub struct Oscillator(dsp_core::oscillator::Oscillator);

impl Oscillator {
    fn next_sample(&mut self) -> f32 {
        self.0.next_sample()
    }
    fn set_freq(&mut self, freq: f32) {
        self.0.set_freq(freq);
    }
}

fn new_oscillator() -> Box<Oscillator> {
    Box::new(Oscillator(dsp_core::oscillator::Oscillator::new(
        dsp_core::oscillator::Waveform::Sine,
    )))
}
