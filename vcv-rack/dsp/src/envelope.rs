use dsp_core::parameter::{ParameterChange, UserParameters};

pub struct Envelope(dsp_core::adsr_envelope::ADSREnvelope);

impl Envelope {
    pub(crate) fn get_releasing(&self) -> bool {
        self.0.get_releasing()
    }

    pub(crate) fn get_release_complete(&self) -> bool {
        self.0.get_release_complete()
    }

    pub(crate) fn set_releasing(&mut self) {
        self.0.set_releasing();
    }

    pub(crate) fn decrement_parameter(&mut self, index: usize) {
        self.0.update_parameter(index, ParameterChange::Decrement);
    }

    pub(crate) fn increment_parameter(&mut self, index: usize) {
        self.0.update_parameter(index, ParameterChange::Increment);
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
