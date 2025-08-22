use std::ops::Div;

use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};

pub struct Analyzer {
    sample_rate: u32,
    scale: f64,
}

impl Analyzer {
    pub fn new(sample_rate: u32, scale: f64) -> Self {
        Analyzer { sample_rate, scale }
    }

    pub fn freq_score(&self, samplebuffer: Vec<f32>) -> anyhow::Result<f64> {
        let spectrum_hann_window = samples_fft_to_spectrum(
            // (windowed) samples
            &samplebuffer,
            // sampling rate
            self.sample_rate,
            // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
            FrequencyLimit::All,
            // optional scale
            Some(&divide_by_N_sqrt),
        )?;
        Ok(spectrum_hann_window
            .data()
            .iter()
            .fold(0., |acc, &(_, val)| acc + val.val() as f64)
            .div(spectrum_hann_window.data().len() as f64)
            * self.scale)
    }
}
