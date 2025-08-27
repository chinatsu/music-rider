use std::path::PathBuf;

use crate::analysis;

use super::get_probe;
use symphonia::core::{audio::SampleBuffer, codecs::DecoderOptions};

/// precompute track
pub fn scan(path: &PathBuf, scale: f64) -> anyhow::Result<Vec<f64>> {
    let mut ret = Vec::new();
    let probed = get_probe(path);
    let mut format = probed.format;
    let track = format.default_track().unwrap();
    let dec_opts: DecoderOptions = Default::default();
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let analyzer = analysis::get_analyzer(analysis::AnalyzerType::Fft, sample_rate, scale);
    if analyzer.is_none() {
        return Ok(ret);
    }
    let analyzer = analyzer.unwrap();

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    let track_id = track.id;
    println!("Scanning track for peaks..");
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let mut sample: SampleBuffer<f32> =
                    SampleBuffer::new(decoded.capacity() as u64, *decoded.spec());
                sample.copy_interleaved_ref(decoded.clone());
                let score = analyzer.freq_score(sample.samples().to_owned())?;
                ret.push(score);
            }
            Err(_) => break,
        }
    }

    Ok(ret)
}
