mod fft_analyzer;

pub enum AnalyzerType {
    Fft,
}

pub trait Analyze {
    fn new(sample_rate: u32, scale: f64) -> Self
    where
        Self: Sized;
    fn freq_score(&self, samplebuffer: Vec<f32>) -> anyhow::Result<f64>;
}

pub fn get_analyzer(
    analyzer_type: AnalyzerType,
    sample_rate: u32,
    scale: f64,
) -> Option<Box<dyn Analyze>> {
    match analyzer_type {
        AnalyzerType::Fft => Some(Box::new(fft_analyzer::FftAnalyzer::new(sample_rate, scale))),
    }
}
