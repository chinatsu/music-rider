use super::Analyze;
use ebur128::{EbuR128, Mode};

pub struct LufsAnalyzer {
    ebur128: EbuR128,
    scale: f64,
}

impl Analyze for LufsAnalyzer {
    fn new(sample_rate: u32, channels: u32, scale: f64) -> anyhow::Result<Self> {
        Ok(LufsAnalyzer {
            ebur128: EbuR128::new(channels, sample_rate, Mode::M)?,
            scale,
        })
    }

    fn freq_score(&mut self, samplebuffer: Vec<f32>) -> anyhow::Result<f64> {
        if !samplebuffer.len().is_power_of_two() {
            return Ok(0.);
        }
        self.ebur128.add_frames_f32(&samplebuffer)?;
        let lufs = self.ebur128.loudness_momentary()?;
        let scaled_lufs = ((lufs + 40.) / 37.).max(0.).min(1.) * self.scale;
        Ok(scaled_lufs)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freq_score() -> anyhow::Result<()> {
        let mut analyzer = LufsAnalyzer::new(44_100, 2, 1.0)?;
        let samplebuffer: Vec<f32> = vec![f32::MAX; 4096];
        let score = analyzer.freq_score(samplebuffer)?;
        println!("score: {score}");
        assert!(score >= 0.);
        Ok(())
    }
}