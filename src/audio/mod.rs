use std::path::PathBuf;

use symphonia::core::{
    audio::{AudioBuffer, RawSampleBuffer, SampleBuffer},
    conv::IntoSample,
    formats::FormatOptions,
    meta::MetadataOptions,
};

pub mod analyze;
use analyze::Analyzer;

mod output;
use output::AudioOutput;

use crate::bike::Bike;

pub struct Audio {
    bike: Option<Bike>,
    directory: PathBuf,
    pub analyzer: Option<Analyzer>,
    pub album_length: usize,
    current_track: usize,
    tracks: Vec<String>,
    audio_output: Option<Box<dyn AudioOutput>>,
}

impl Audio {
    pub fn new(directory: PathBuf, bike: Option<Bike>) -> Self {
        let mut audio = Audio {
            bike: bike,
            analyzer: None,
            directory: directory.clone(),
            album_length: 0,
            current_track: 0,
            tracks: Vec::new(),
            audio_output: None,
        };
        audio.tracks = audio.files();
        audio.album_length = audio.tracks.len();
        audio
    }

    fn files(&self) -> Vec<String> {
        let mut flacs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.directory) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "flac" {
                        if let Some(name) = entry.path().file_name() {
                            if let Some(name_str) = name.to_str() {
                                flacs.push(name_str.to_string());
                            }
                        }
                    }
                }
            }
        }
        flacs.sort();
        flacs
    }

    pub fn next_track(&mut self) -> Option<String> {
        if self.current_track < self.album_length {
            self.current_track += 1;
            let track = self.tracks.get(self.current_track).cloned();
            track
        } else {
            None
        }
    }

    pub fn reset_analyzer(&mut self) {
        if let Some(analyzer) = &mut self.analyzer {
            analyzer.reset();
            self.analyzer = None;
        }
    }

    pub async fn play_track(&mut self) -> anyhow::Result<usize> {
        let path = self.directory.join(&self.tracks[self.current_track]);
        println!("Playing track: {}", path.display());
        let src = std::fs::File::open(&path).expect("failed to open media");
        let mss = symphonia::core::io::MediaSourceStream::new(Box::new(src), Default::default());
        let mut hint = symphonia::core::probe::Hint::new();
        hint.with_extension("flac");
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .expect("unsupported format");

        let mut format = probed.format;
        let track = match format
            .tracks()
            .iter()
            .find(|track| track.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        {
            Some(track) => track.clone(),
            _ => return Ok(0),
        };
        if self.analyzer.is_none() {
            self.analyzer = Some(Analyzer::new(
                track.codec_params.channels.iter().len() as u32,
                track.codec_params.sample_rate.unwrap(),
            ));
        }

        let dec_opts: symphonia::core::codecs::DecoderOptions = Default::default();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .expect("unsupported codec");
        let track_id = track.id;
        let tb = track.codec_params.time_base;

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    unimplemented!();
                }
                Err(err) => {
                    println!("Error reading packet: {}", err);
                    return Ok(0);
                }
            };

            while !format.metadata().is_latest() {
                format.metadata().pop();
            }

            if packet.track_id() != track_id {
                println!(
                    "oops! Track ID mismatch: expected {}, got {}",
                    track_id,
                    packet.track_id()
                );
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    if self.audio_output.is_none() {
                        let spec = *decoded.spec();
                        let duration = decoded.capacity() as u64;
                        self.audio_output
                            .replace(output::try_open(spec, duration).unwrap());
                    }

                    if let Some(analyzer) = &mut self.analyzer {
                        let mut sample: SampleBuffer<f32> =
                            SampleBuffer::new(decoded.capacity() as u64, decoded.spec().clone());
                        sample.copy_interleaved_ref(decoded.clone());
                        analyzer.add_frames(sample)?;
                    }

                    if let Some(audio_output) = self.audio_output.as_mut() {
                        audio_output.write(decoded).unwrap();
                    }
                    if let Some(tb) = tb {
                        let t = tb.calc_time(packet.ts());

                        let secs = t.seconds as f64 + t.frac;
                        if secs as u64 % 3 == 0 && format!("{:.1}", secs).ends_with("0") {
                            if let Some(analyzer) = &self.analyzer {
                                let loudness = analyzer.get_loudness()?;
                                if let Some(bike) = &self.bike {
                                    bike.set_level_from_loudness(loudness).await?;
                                    bike.print_stats().await?;
                                }
                            }
                        }
                    }
                }
                Err(symphonia::core::errors::Error::IoError(_)) => continue,
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(err) => {
                    println!("{}", err);
                    return Ok(0);
                }
            }
        }
    }
}
