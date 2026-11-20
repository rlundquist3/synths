#[derive(Debug, Clone)]
pub struct Material {
    pub mp_id: &'static str,
    pub name: &'static str,
    pub peaks_by_temp: &'static [PeaksByTemp],
}

#[derive(Debug, Clone)]
pub struct PeaksByTemp {
    pub temp: u16, // 0 == athermal, else temp in K
    pub formatted_temp: &'static str,
    pub peaks_by_species: &'static [PeaksBySpecies],
}

#[derive(Debug, Clone)]
pub struct PeaksBySpecies {
    pub species: &'static str,
    pub peaks: &'static [Peak],
}

#[derive(Debug, Clone, Copy)]
pub struct Peak {
    pub freq_ratio: f32,
    pub width_ratio: f32,
    pub density: f32,
    // TODO: consider adding reference frequency (that the ratio came from)
}
