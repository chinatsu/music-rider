use ebur128::{EbuR128, Mode};
use symphonia::core::audio::SampleBuffer;

pub struct Analyzer {
    ebu: EbuR128,
}

impl Analyzer {
    pub fn new(channels: u32, samplerate: u32) -> Self {
        Analyzer {
            ebu: EbuR128::new(channels, samplerate, Mode::M).unwrap(),
        }
    }

    pub fn add_frames(&mut self, samples: SampleBuffer<f32>) -> anyhow::Result<()> {
        self.ebu.add_frames_f32(samples.samples())?;
        Ok(())
    }

    pub fn get_loudness(&self) -> anyhow::Result<f64> {
        let ebu = self.ebu.loudness_momentary()?;
        Ok(ebu)
    }

    pub fn reset(&mut self) {
        self.ebu.reset();
    }
}