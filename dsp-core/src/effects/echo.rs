use crate::{
    DEFAULT_SAMPLE_RATE,
    buffer_pool::BufferPool,
    effects::Effect,
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};
use alloc::vec;
use alloc::vec::Vec;

const MAX_DELAY_SECS: f32 = 1.0;

#[derive(Debug)]
pub struct Echo {
    sample_rate: u32,
    on: bool,
    buffer: &'static mut [f32],
    write_index: usize,
    parameters: Vec<Parameter>,
}

impl Echo {
    pub fn new(delay_secs: f32, amp: f32, buffer_pool: &mut BufferPool) -> Self {
        let sample_rate = DEFAULT_SAMPLE_RATE;
        let max_samples = (MAX_DELAY_SECS * sample_rate as f32) as usize;

        Echo {
            sample_rate,
            on: false,
            buffer: buffer_pool.take(max_samples),
            write_index: 0,
            parameters: vec![
                Parameter::new("Delay", delay_secs, 0.1, (0.0, 1.0)),
                Parameter::new("Amp", amp, 0.1, (0.0, 1.0)),
            ],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }
}

impl Effect for Echo {
    fn toggle(&mut self) {
        self.on = !self.on;
    }

    fn is_on(&self) -> bool {
        self.on
    }

    fn process(&mut self, sample: f32) -> f32 {
        if !self.on {
            return sample;
        }

        let delay_samples = (self.parameters[0].get_value() * self.sample_rate as f32) as usize;
        let amp = self.parameters[1].get_value();

        let max = self.buffer.len();
        let read_index = (self.write_index + max - delay_samples.min(max - 1)) % max;
        let delayed = self.buffer[read_index];

        let result = sample - amp * delayed;
        self.buffer[self.write_index] = result;
        self.write_index = (self.write_index + 1) % max;

        result
    }

    fn get_name(&self) -> &'static str {
        "Echo"
    }
}

impl UserParameters for Echo {
    fn get_parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    fn set_parameter(&mut self, index: usize, value: f32) {
        if let Some(param) = self.parameters.get_mut(index) {
            param.set_value(value.clamp(param.range.0, param.range.1))
        }
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) {
        if let Some(param) = self.parameters.get_mut(index) {
            let delta = match change {
                Increment => param.delta,
                Decrement => -param.delta,
            };
            param.set_value((param.get_value() + delta).clamp(param.range.0, param.range.1));
        }
    }
}
