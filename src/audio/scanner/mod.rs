use std::path::PathBuf;

use crate::analysis;

use super::get_probe;
use symphonia::core::{audio::SampleBuffer, codecs::DecoderOptions, meta::Value};

/// precompute track
pub fn scan(path: &PathBuf, scale: f64, analyzer_choice: String) -> anyhow::Result<Vec<(Option<u8>, f64)>> {
    let mut ret = Vec::new();
    let probed = get_probe(path);
    let mut format = probed.format;
    let bpm = if let Some(bpm) = format.metadata().current().unwrap().tags().iter().find(|i| i.key == "BPM") {
        match &bpm.value {
            Value::String(val) => Some(val.clone().parse::<u8>().unwrap()),
            _ => None
        }
    } else {
        None
    };
    let track = format.default_track().unwrap();
    
    let dec_opts: DecoderOptions = Default::default();
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = if let Some(layout) = track.codec_params.channel_layout {
        match layout {
            symphonia::core::audio::Layout::Mono => 1,
            symphonia::core::audio::Layout::Stereo => 2,
            symphonia::core::audio::Layout::TwoPointOne => 3,
            symphonia::core::audio::Layout::FivePointOne => 6,
        }
    } else {
        2
    };

    let analyzer_type = match analyzer_choice.as_str() {
        "fft" => analysis::AnalyzerType::Fft,
        _ => analysis::AnalyzerType::Lufs
    };

    let mut analyzer = analysis::get_analyzer(analyzer_type, sample_rate, channels, scale)?;

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
                ret.push((bpm, score));
            }
            Err(_) => break,
        }
    }

    Ok(ret)
}
