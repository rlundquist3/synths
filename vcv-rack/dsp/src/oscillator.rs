use crate::ffi::SamplesByWaveform;

pub struct Oscillator(dsp_core::oscillator::Oscillator);

impl Oscillator {
    pub(crate) fn next_sample_by_waveform(&mut self) -> SamplesByWaveform {
        let s = self.0.next_sample_by_waveform();
        SamplesByWaveform {
            sine: s.sine,
            sawtooth: s.sawtooth,
            square: s.square,
            triangle: s.triangle,
        }
    }

    pub(crate) fn set_sample_rate(&mut self, sample_rate: u32) {
        self.0.set_sample_rate(sample_rate);
    }

    pub(crate) fn set_freq(&mut self, freq: f32) {
        self.0.set_freq(freq);
    }
}

pub(crate) fn new_oscillator() -> Box<Oscillator> {
    Box::new(Oscillator(dsp_core::oscillator::Oscillator::new(
        dsp_core::oscillator::Waveform::Sine,
    )))
}
