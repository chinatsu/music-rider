use std::ops::Div;

use spectrum_analyzer::scaling::scale_to_zero_to_one;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};

pub struct Analyzer {}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {}
    }

    pub fn low_freq_score(&self, samplebuffer: Vec<f32>) -> anyhow::Result<f64> {
        if samplebuffer.len() < 2048 {
            return Err(anyhow::anyhow!("Not enough samples to analyze"));
        }
        let hann_window = hann_window(&samplebuffer[0..2048]);
        let spectrum_hann_window = samples_fft_to_spectrum(
            // (windowed) samples
            &hann_window,
            // sampling rate
            44100,
            // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
            FrequencyLimit::Range(0., 150.),
            // optional scale
            Some(&scale_to_zero_to_one),
        )?;
        Ok(spectrum_hann_window
            .data()
            .iter()
            .fold(0., |acc, &(_, val)| acc + val.val() as f64)
            .div(spectrum_hann_window.data().len() as f64))
    }
}
