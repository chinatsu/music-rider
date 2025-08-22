use std::{path::PathBuf, sync::mpsc::Sender};
use symphonia::core::{
    audio::SampleBuffer, formats::FormatOptions, meta::MetadataOptions, probe::ProbeResult,
};

pub mod analyze;

mod output;
use output::AudioOutput;

pub struct Audio {
    path: PathBuf,
    pub album_length: usize,
    current_track: usize,
    tracks: Vec<PathBuf>,
    audio_output: Option<Box<dyn AudioOutput>>,
}

impl Audio {
    pub fn new(path: PathBuf) -> Self {
        let mut audio = Audio {
            path: path.clone(),
            album_length: 0,
            current_track: 0,
            tracks: Vec::new(),
            audio_output: None,
        };
        audio.tracks = audio.files();
        audio.album_length = audio.tracks.len();
        audio
    }

    fn files(&self) -> Vec<PathBuf> {
        let mut flacs = Vec::new();
        if self.path.is_file() {
            return vec![self.path.clone()];
        }
        if let Ok(entries) = std::fs::read_dir(&self.path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension()
                    && ext == "flac"
                    && let Some(name) = entry.path().file_name()
                    && let Some(name_str) = name.to_str()
                {
                    flacs.push(self.path.join(name_str));
                }
            }
        }
        flacs.sort();
        flacs
    }

    pub fn next_track(&mut self) -> Option<PathBuf> {
        if self.current_track < self.album_length {
            self.current_track += 1;

            self.tracks.get(self.current_track).cloned()
        } else {
            None
        }
    }

    pub fn flush(&mut self) {
        if let Some(output) = &mut self.audio_output {
            output.flush();
            self.audio_output = None;
        }
    }

    pub fn play_track(&mut self, sender: Sender<Vec<f32>>) -> anyhow::Result<usize> {
        let probed = get_probe(&self.tracks[self.current_track]);
        let mut format = probed.format;
        let track = match format
            .tracks()
            .iter()
            .find(|track| track.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        {
            Some(track) => track.clone(),
            _ => return Ok(0),
        };

        let dec_opts: symphonia::core::codecs::DecoderOptions = Default::default();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .expect("unsupported codec");
        let track_id = track.id;

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    unimplemented!();
                }
                Err(_) => {
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

                    if let Some(audio_output) = self.audio_output.as_mut() {
                        audio_output.write(decoded.clone()).unwrap();
                    }

                    let mut sample: SampleBuffer<f32> =
                        SampleBuffer::new(decoded.capacity() as u64, *decoded.spec());
                    sample.copy_interleaved_ref(decoded.clone());
                    let _ = sender.send(sample.samples().to_vec());
                }
                Err(symphonia::core::errors::Error::IoError(_)) => continue,
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(err) => {
                    println!("{err}");
                    return Ok(0);
                }
            }
        }
    }
}

pub fn get_probe(path: &PathBuf) -> ProbeResult {
    let src = std::fs::File::open(path).expect("failed to open media");
    let mss = symphonia::core::io::MediaSourceStream::new(Box::new(src), Default::default());
    let mut hint = symphonia::core::probe::Hint::new();
    hint.with_extension("flac");
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();
    symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format")
}

pub fn get_flac_from_dir(path: PathBuf) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.clone());
    }
    if let Ok(entries) = std::fs::read_dir(&path) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension()
                && ext == "flac"
                && let Some(name) = entry.path().file_name()
                && let Some(name_str) = name.to_str()
            {
                return Some(path.join(name_str));
            }
        }
    }
    None
}
