use core::sync::atomic::AtomicBool;

use alloc::sync::Arc;
use heapless::Vec;

use crate::DEFAULT_SAMPLE_RATE;
use crate::adsr_envelope::ADSREnvelope;
use crate::engines::singing_materials::material::{
    MATERIAL_COUNT, MATERIALS, Material, OSCILLATOR_COUNT,
};
use crate::midi::get_linear_bent_freq;
use crate::oscillator::{Oscillator, Waveform};
use crate::voices::Voice;

pub struct SingingMaterialsVoice<'a> {
    sample_rate: u32,
    freq: f32,
    midi_note: usize,
    oscillators: Vec<Oscillator, OSCILLATOR_COUNT>,
    materials: Vec<Material<'a>, MATERIAL_COUNT>,
    material_index: usize,

    pub envelope: Option<ADSREnvelope>,
    pub on: Arc<AtomicBool>,
}

impl<'a> SingingMaterialsVoice<'a> {
    pub fn new(envelope: Option<ADSREnvelope>) -> Self {
        let osc = Oscillator::new(Waveform::Sine);
        let mut oscillators: Vec<Oscillator, OSCILLATOR_COUNT> = Vec::new();
        for _ in 0..OSCILLATOR_COUNT {
            oscillators.push(osc.clone()).ok();
        }

        SingingMaterialsVoice {
            sample_rate: DEFAULT_SAMPLE_RATE,
            freq: 0.0,
            midi_note: 0,
            oscillators,
            materials: MATERIALS,
            material_index: 0,
            envelope,
            on: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_material(&mut self, index: usize) {
        self.material_index = index;
    }

    pub fn get_material_name(self) -> &'a str {
        self.materials[self.material_index].name
    }

    pub fn set_waveform(&mut self, osc_index: usize, waveform: Waveform) {
        self.oscillators[osc_index].set_waveform(waveform);
    }

    pub fn set_offset(&mut self, osc_index: usize, offset: f32) {
        let phase = self.oscillators[osc_index].phase;
        self.oscillators[osc_index].set_phase(phase + offset);
    }

    pub fn set_releasing(&mut self) {
        if let Some(envelope) = &mut self.envelope {
            envelope.set_releasing();
        }
    }

    pub fn get_releasing(&self) -> bool {
        match &self.envelope {
            Some(envelope) => envelope.get_releasing(),
            None => false,
        }
    }

    pub fn get_release_complete(&self) -> bool {
        match &self.envelope {
            Some(envelope) => envelope.get_release_complete(),
            None => false,
        }
    }
}

impl<'a> Clone for SingingMaterialsVoice<'a> {
    fn clone(&self) -> Self {
        SingingMaterialsVoice {
            sample_rate: self.sample_rate,
            freq: self.freq,
            midi_note: self.midi_note,
            oscillators: self.oscillators.clone(),
            materials: self.materials.clone(),
            material_index: self.material_index,
            envelope: self.envelope.clone(),
            on: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<'a> Voice for SingingMaterialsVoice<'a> {
    fn set_freq(&mut self, freq: f32, midi_note: usize) {
        for i in 0..OSCILLATOR_COUNT {
            self.oscillators[i].set_freq(freq * self.materials[self.material_index].ratios[i]);
        }
        self.midi_note = midi_note;
    }

    fn set_pitch_bend(&mut self, midi_bend: u16) {
        let bent_freq = get_linear_bent_freq(self.midi_note, midi_bend, 2);
        self.set_freq(bent_freq, self.midi_note);
    }
}

impl<'a> Iterator for SingingMaterialsVoice<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let envelope_amp = self
            .envelope
            .as_mut()
            .and_then(|envelope| envelope.next())
            .unwrap_or(1.0);

        let mut sample = 0.0;
        for i in 0..OSCILLATOR_COUNT {
            sample +=
                self.materials[self.material_index].amps[i] * self.oscillators[i].next_sample();
        }

        // clamp should be unnecessary, but here for safety
        Some(envelope_amp * sample.clamp(-1.0, 1.0))
    }
}
