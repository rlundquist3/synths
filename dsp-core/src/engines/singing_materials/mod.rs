use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::Ordering;

use crate::DEFAULT_SAMPLE_RATE;
use crate::effects::gain::Gain;
use crate::engines::Engine;
use crate::engines::singing_materials::material::OSCILLATOR_COUNT;
use crate::engines::singing_materials::voice::SingingMaterialsVoice;
use crate::midi::MIDI_NOTE_FREQS;
use crate::oscillator::Waveform;
use crate::parameter::ParameterChange::{self, Decrement, Increment};
use crate::parameter::UserParameters;
use crate::voices::{Voice, Voices};
use crate::{adsr_envelope::ADSREnvelope, effects::Effect, parameter::Parameter};

pub mod material;
pub mod voice;

const VOICE_COUNT: usize = 6;

pub struct SingingMaterials<'a> {
    sample_rate: u32,
    pub voices: Voices<SingingMaterialsVoice<'a>>,
    headroom_gain: Gain,
    envelope: ADSREnvelope,
    parameters: Vec<Parameter>,
}

impl<'a> SingingMaterials<'a> {
    pub fn new() -> Self {
        let envelope = ADSREnvelope::new(0.3, 0.3, 0.9, 0.2);
        let voice = SingingMaterialsVoice::new(Some(envelope.clone()));
        let voices = Voices::new((0..VOICE_COUNT).map(|_| voice.clone()).collect());

        let mut headroom_gain = Gain::new(-16.0);
        headroom_gain.toggle();

        SingingMaterials {
            sample_rate: DEFAULT_SAMPLE_RATE,
            voices,
            headroom_gain,
            envelope,
            parameters: vec![
                Parameter::new("Material", 0.0, 1.0, (0.0, 3.0)),
                Parameter::new("Wave", 1.0, 1.0, (1.0, 4.0)),
            ],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.voices.voices.iter_mut().for_each(|voice| {
            voice.set_sample_rate(sample_rate);
        });
    }
}

impl<'a> Engine for SingingMaterials<'a> {
    fn note_on(&mut self, note: u8) {
        let voice = self.voices.voice_on(note);
        voice.set_freq(MIDI_NOTE_FREQS[note as usize], note as usize);
        voice.on.store(true, Ordering::Relaxed);
    }

    fn note_off(&mut self, note: u8) {
        if let Some(voice) = self.voices.voice_off(note) {
            voice.set_releasing();
            voice.on.store(false, Ordering::Relaxed);
        }
    }

    fn set_pitch_bend(&mut self, bend: u16) {
        self.voices.voices.iter_mut().for_each(|voice| {
            voice.set_pitch_bend(bend);
        });
    }

    fn get_envelope_parameters(&self) -> &[Parameter] {
        self.envelope.get_parameters()
    }

    fn get_name(&self) -> &'static str {
        "Singing Materials"
    }
}

impl<'a> Iterator for SingingMaterials<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.voices.next()?;
        let sample = self.headroom_gain.process(raw).clamp(-1.0, 1.0);

        Some(sample)
    }
}

impl<'a> Iterator for Voices<SingingMaterialsVoice<'a>> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.voices.iter_mut().fold(0.0, |acc: f32, voice| {
            if voice.on.load(Ordering::Relaxed) || !voice.get_release_complete() {
                acc + voice.next().unwrap_or(0.0)
            } else {
                acc
            }
        }))
    }
}

impl<'a> UserParameters for SingingMaterials<'a> {
    fn get_parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    fn set_parameter(&mut self, index: usize, value: f32) {
        let synth_param_count = self.parameters.len();

        if index >= synth_param_count {
            self.voices.voices.iter_mut().for_each(|voice| {
                if let Some(envelope) = &mut voice.envelope {
                    envelope.set_parameter(index - synth_param_count, value);
                }
            });
            self.envelope
                .set_parameter(index - synth_param_count, value);
        } else if let Some(param) = self.parameters.get_mut(index) {
            let value = value.clamp(param.range.0, param.range.1);
            param.set_value(value);

            match index {
                0 => {
                    self.voices.voices.iter_mut().for_each(|voice| {
                        voice.set_material(value as usize);
                    });
                }
                1 => {
                    let waveform = match value {
                        1.0 => Waveform::Sine,
                        2.0 => Waveform::Sawtooth,
                        3.0 => Waveform::Square,
                        4.0 => Waveform::Triangle,
                        _ => Waveform::Sine,
                    };
                    self.voices.voices.iter_mut().for_each(|voice| {
                        for i in 0..OSCILLATOR_COUNT {
                            voice.set_waveform(i, waveform.clone());
                        }
                    });
                }
                _ => {}
            }
        }
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) {
        let synth_param_count = self.parameters.len();

        if index >= synth_param_count {
            self.voices.voices.iter_mut().for_each(|voice| {
                if let Some(envelope) = &mut voice.envelope {
                    envelope.update_parameter(index - synth_param_count, change.clone());
                }
            });
            self.envelope
                .update_parameter(index - synth_param_count, change.clone());
        } else if let Some(param) = self.parameters.get_mut(index) {
            let delta = match change {
                Increment => param.delta,
                Decrement => -param.delta,
            };
            let updated_value = (param.get_value() + delta).clamp(param.range.0, param.range.1);

            self.set_parameter(index, updated_value);
        }
    }
}
