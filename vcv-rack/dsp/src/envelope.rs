use crate::ffi::EnvelopeParameters;
use dsp_core::parameter::UserParameters;

pub struct Envelope(dsp_core::adsr_envelope::ADSREnvelope);

impl Envelope {
    pub(crate) fn set_sample_rate(&mut self, sample_rate: u32) {
        self.0.set_sample_rate(sample_rate);
    }

    pub(crate) fn get_releasing(&self) -> bool {
        self.0.get_releasing()
    }

    pub(crate) fn get_release_complete(&self) -> bool {
        self.0.get_release_complete()
    }

    pub(crate) fn set_releasing(&mut self) {
        self.0.set_releasing();
    }

    pub(crate) fn set_parameter(&mut self, index: usize, value: f32) {
        self.0.set_parameter(index, value);
    }

    pub(crate) fn get_parameter_values(&self) -> EnvelopeParameters {
        EnvelopeParameters {
            attack: self.0.get_parameters()[0].get_value(),
            decay: self.0.get_parameters()[1].get_value(),
            sustain: self.0.get_parameters()[2].get_value(),
            release: self.0.get_parameters()[3].get_value(),
        }
    }

    pub(crate) fn next(&mut self) -> f32 {
        self.0.next().unwrap_or(0.0)
    }
}

pub(crate) fn new_envelope() -> Box<Envelope> {
    Box::new(Envelope(dsp_core::adsr_envelope::ADSREnvelope::new(
        0.8, 0.2, 0.8, 0.3,
    )))
}
