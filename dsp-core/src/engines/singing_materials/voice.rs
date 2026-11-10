use core::sync::atomic::AtomicBool;

use alloc::sync::Arc;
use heapless::Vec;

use crate::DEFAULT_SAMPLE_RATE;
use crate::adsr_envelope::ADSREnvelope;
use crate::midi::get_linear_bent_freq;
use crate::oscillator::{Oscillator, Waveform};
use crate::voices::Voice;

pub const OSCILLATOR_COUNT: usize = 6;
pub const MATERIAL_COUNT: usize = 4;

#[derive(Debug, Clone)]
struct Material<'a> {
    name: &'a str,
    ratios: [f32; OSCILLATOR_COUNT],
    amps: [f32; OSCILLATOR_COUNT], // amps must add up to 1
}

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

        let materials: Vec<Material, MATERIAL_COUNT> = Vec::from_array([
            Material {
                name: "Diamond (athermal)",
                ratios: [
                    1.0,
                    0.6176855363089343,
                    0.8242325452945736,
                    0.8801217359612759,
                    0.9408708562511697,
                    1.0615591085604257,
                ],
                amps: [
                    0.3664732953505933,
                    0.09141874712164959,
                    0.10325275256154441,
                    0.07951860819996766,
                    0.20119166718222947,
                    0.1581449295840154,
                ],
            },
            Material {
                name: "Diamond (300K)",
                ratios: [
                    1.0,
                    0.5692976384462044,
                    0.43900112570724087,
                    0.34670776251714175,
                    0.21641124977817824,
                    0.10964049628374992,
                ],
                amps: [
                    0.30013822873101764,
                    0.18633916932068376,
                    0.1620029032663612,
                    0.15150614756972558,
                    0.11295217081754953,
                    0.08706138029466234,
                ],
            },
            Material {
                name: "Calcium Carbonate (athermal)",
                ratios: [
                    1.0,
                    0.1811930712502019,
                    0.4284349502687911,
                    1.2152458711455953,
                    1.567201957748528,
                    2.073320627739523,
                ],
                amps: [
                    0.28342397313947865,
                    0.07066137228035566,
                    0.1619812874223158,
                    0.19312025783520717,
                    0.21673573007947727,
                    0.07407737924316543,
                ],
            },
            Material {
                name: "Stishovite (athermal)",
                ratios: [
                    1.0,
                    0.7819028890104764,
                    1.2663080513135236,
                    1.5647567295097142,
                    1.8677969258319997,
                    2.0858940368215233,
                ],
                amps: [
                    0.2658995291617924,
                    0.16431379706101254,
                    0.1561619530226568,
                    0.1610521238655463,
                    0.14463332857882436,
                    0.10793926831016763,
                ],
            },
        ]);

        SingingMaterialsVoice {
            sample_rate: DEFAULT_SAMPLE_RATE,
            freq: 0.0,
            midi_note: 0,
            oscillators,
            materials,
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
