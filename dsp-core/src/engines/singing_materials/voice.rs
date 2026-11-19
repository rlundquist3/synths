use core::sync::atomic::AtomicBool;

use alloc::sync::Arc;
use heapless::Vec;

use crate::DEFAULT_SAMPLE_RATE;
use crate::adsr_envelope::ADSREnvelope;
use crate::engines::singing_materials::data::MATERIALS;
use crate::engines::singing_materials::types::{PeaksBySpecies, PeaksByTemp};
use crate::midi::get_linear_bent_freq;
use crate::oscillator::{Oscillator, Waveform};
use crate::voices::Voice;

pub const OSCILLATOR_COUNT: usize = 9;

/*
TODO:
- add temperature and species isolation parameters
- play with the width data to inform per-oscillator vibrato
- play with different envelopes per oscillator (bit of a refactor, but could sound interesting)
*/

pub struct SingingMaterialsVoice<'a> {
    sample_rate: u32,
    freq: f32,
    midi_note: usize,
    oscillators: Vec<Oscillator, OSCILLATOR_COUNT>,
    material_index: usize,
    temp: u16,
    species: &'a str,
    peaks_by_temp: &'a PeaksByTemp,
    peaks_by_species: &'a PeaksBySpecies,

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

        let peaks_by_temp = MATERIALS[0]
            .peaks_by_temp
            .iter()
            .find(|p| p.temp == 0)
            .unwrap();
        let peaks_by_species = peaks_by_temp
            .peaks_by_species
            .iter()
            .find(|p| p.species == "total")
            .unwrap();

        SingingMaterialsVoice {
            sample_rate: DEFAULT_SAMPLE_RATE,
            freq: 0.0,
            midi_note: 0,
            oscillators,
            material_index: 0,
            temp: 0,
            species: "total",
            peaks_by_temp,
            peaks_by_species,
            envelope,
            on: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.oscillators.iter_mut().for_each(|oscillator| {
            oscillator.set_sample_rate(sample_rate);
        });
    }

    pub fn set_material(&mut self, index: usize) {
        if index < MATERIALS.len() {
            self.material_index = index;
            self.set_temp(self.temp);
        }
    }

    pub fn set_temp(&mut self, temp: u16) {
        if let Some(peaks_by_temp) = MATERIALS[self.material_index]
            .peaks_by_temp
            .iter()
            .find(|p| p.temp == temp)
        {
            self.temp = temp;
            self.peaks_by_temp = peaks_by_temp;
            self.set_species(self.species);
        }
    }

    pub fn set_species(&mut self, species: &'a str) {
        if let Some(peaks_by_species) = self
            .peaks_by_temp
            .peaks_by_species
            .iter()
            .find(|p| p.species == species)
        {
            self.species = species;
            self.peaks_by_species = peaks_by_species;
        }
    }

    pub fn get_material_name(self) -> &'static str {
        MATERIALS[self.material_index].name
    }

    pub fn get_current_temp(&self) -> &'static str {
        self.peaks_by_temp.formatted_temp
    }

    pub fn get_current_species(&self) -> &str {
        self.species
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
            material_index: self.material_index,
            temp: self.temp,
            species: self.species,
            peaks_by_temp: self.peaks_by_temp,
            peaks_by_species: self.peaks_by_species,
            envelope: self.envelope.clone(),
            on: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<'a> Voice for SingingMaterialsVoice<'a> {
    fn set_freq(&mut self, freq: f32, midi_note: usize) {
        for i in 0..OSCILLATOR_COUNT {
            if self.peaks_by_species.peaks.len() > i {
                self.oscillators[i].set_freq(freq * self.peaks_by_species.peaks[i].freq_ratio);
            }
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
            if self.peaks_by_species.peaks.len() > i {
                sample +=
                    self.peaks_by_species.peaks[i].density * self.oscillators[i].next_sample();
            }
        }

        // clamp should be unnecessary, but here for safety
        Some(envelope_amp * sample.clamp(-1.0, 1.0))
    }
}
