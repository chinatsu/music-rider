mod fft_analyzer;
mod lufs_analyzer;

pub enum AnalyzerType {
    Fft,
    Lufs,
}

pub trait Analyze {
    fn new(sample_rate: u32, channels: u32, scale: f64) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn freq_score(&mut self, samplebuffer: Vec<f32>) -> anyhow::Result<f64>;
}

pub fn get_analyzer(
    analyzer_type: AnalyzerType,
    sample_rate: u32,
    channels: u32,
    scale: f64,
) -> anyhow::Result<Box<dyn Analyze>> {
    match analyzer_type {
        AnalyzerType::Fft => Ok(Box::new(fft_analyzer::FftAnalyzer::new(sample_rate, channels, scale)?)),
        AnalyzerType::Lufs => Ok(Box::new(lufs_analyzer::LufsAnalyzer::new(sample_rate, channels, scale)?))
    }
}
