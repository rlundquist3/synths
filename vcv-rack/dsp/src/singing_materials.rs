use dsp_core::{
    engines::singing_materials::voice::SingingMaterialsVoice, oscillator::Waveform, voices::Voice,
};

pub struct SingingMaterials<'a>(
    dsp_core::engines::singing_materials::voice::SingingMaterialsVoice<'a>,
);

impl<'a> SingingMaterials<'a> {
    pub(crate) fn next_sample(&mut self) -> f32 {
        self.0.next().unwrap_or(0.0)
    }

    pub(crate) fn set_freq(&mut self, freq: f32) {
        self.0.set_freq(freq, 0);
    }

    pub(crate) fn set_material(&mut self, index: usize) {
        self.0.set_material(index);
    }

    // pub(crate) fn get_material_name(&self) -> &'a str {
    //     self.0.get_material_name()
    // }

    pub(crate) fn set_waveform(&mut self, index: usize) {
        let waveform = match index {
            0 => Waveform::Sine,
            1 => Waveform::Sawtooth,
            2 => Waveform::Square,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        };

        for i in 0..1 {
            self.0.set_waveform(i, waveform);
        }
    }
}

pub(crate) unsafe fn new_singing_materials<'a>() -> Box<SingingMaterials<'a>> {
    Box::new(SingingMaterials(SingingMaterialsVoice::new()))
}
