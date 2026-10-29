use cxx::CxxVector;
use ffi::*;

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
}

pub struct Oscillator(dsp_core::oscillator::Oscillator);

impl Oscillator {
    fn next_sample_by_waveform(&mut self) -> SamplesByWaveform {
        let s = self.0.next_sample_by_waveform();
        SamplesByWaveform {
            sine: s.sine,
            sawtooth: s.sawtooth,
            square: s.square,
            triangle: s.triangle,
        }
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
