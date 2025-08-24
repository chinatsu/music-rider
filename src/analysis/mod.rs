mod fft_analyzer;

pub trait Analyze {
    fn new(sample_rate: u32, scale: f64) -> Self
    where
        Self: Sized;
    fn freq_score(&self, samplebuffer: Vec<f32>) -> anyhow::Result<f64>;
}

pub fn get_analyzer(name: &str, sample_rate: u32, scale: f64) -> Option<Box<dyn Analyze>> {
    match name {
        "fft" => Some(Box::new(fft_analyzer::FftAnalyzer::new(sample_rate, scale))),
        _ => None,
    }
}
