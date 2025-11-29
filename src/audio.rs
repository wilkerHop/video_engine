use anyhow::{Context, Result};
use hound;
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Decodes audio files into raw samples (f32, interleaved)
pub struct AudioDecoder;

impl AudioDecoder {
    /// Decode an audio file to a vector of samples (f32)
    /// Returns (samples, sample_rate, channels)
    pub fn decode(path: &Path) -> Result<(Vec<f32>, u32, u32)> {
        let src = File::open(path).context("Failed to open audio file")?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .context("Unsupported audio format")?;

        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .context("No supported audio track found")?;

        let dec_opts: DecoderOptions = Default::default();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .context("Unsupported codec")?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.unwrap_or_default().count() as u32;

        let mut all_samples = Vec::new();

        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut sample_buf =
                        SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                    sample_buf.copy_interleaved_ref(decoded);
                    all_samples.extend_from_slice(sample_buf.samples());
                }
                Err(e) => {
                    eprintln!("Error decoding packet: {}", e);
                    break;
                }
            }
        }

        Ok((all_samples, sample_rate, channels))
    }
}

/// Mixes multiple audio tracks
pub struct AudioMixer {
    output_sample_rate: u32,
    output_channels: u32,
    tracks: Vec<MixedTrack>,
}

struct MixedTrack {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u32,
    start_time: f32,
    volume: f32,
}

impl AudioMixer {
    pub fn new(sample_rate: u32, channels: u32) -> Self {
        Self {
            output_sample_rate: sample_rate,
            output_channels: channels,
            tracks: Vec::new(),
        }
    }

    pub fn add_track(
        &mut self,
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u32,
        start_time: f32,
        volume: f32,
    ) {
        self.tracks.push(MixedTrack {
            samples,
            sample_rate,
            channels,
            start_time,
            volume,
        });
    }

    /// Mix all tracks into a single buffer
    pub fn mix(&self, duration_seconds: f32) -> Vec<f32> {
        let total_samples = (duration_seconds * self.output_sample_rate as f32) as usize
            * self.output_channels as usize;
        let mut mixed_buffer = vec![0.0; total_samples];

        for track in &self.tracks {
            // Simple resampling (nearest neighbor) and mixing
            // NOTE: For production, use a proper resampler like `rubato`

            let start_sample = (track.start_time * self.output_sample_rate as f32) as usize
                * self.output_channels as usize;

            // Ratio between track sample rate and output sample rate
            let rate_ratio = track.sample_rate as f32 / self.output_sample_rate as f32;

            for (i, sample) in mixed_buffer.iter_mut().enumerate() {
                if i < start_sample {
                    continue;
                }

                let track_index = i - start_sample;
                // Map output sample index to input sample index based on rate
                // We process interleaved samples, so we need to be careful with channels

                let frame_index = track_index / self.output_channels as usize;
                let channel_index = track_index % self.output_channels as usize;

                let input_frame_index = (frame_index as f32 * rate_ratio) as usize;

                // Handle channel mapping (mono to stereo, etc.)
                let input_channel_index = if track.channels == 1 {
                    0 // Use the single channel for all output channels
                } else {
                    channel_index % track.channels as usize
                };

                let input_sample_index =
                    input_frame_index * track.channels as usize + input_channel_index;

                if input_sample_index < track.samples.len() {
                    *sample += track.samples[input_sample_index] * track.volume;
                }
            }
        }

        // Hard clipping prevention (tanh soft clipping)
        for sample in &mut mixed_buffer {
            *sample = sample.tanh();
        }

        mixed_buffer
    }

    /// Export mixed audio to WAV file
    pub fn export(&self, path: &Path, samples: &[f32]) -> Result<()> {
        let spec = hound::WavSpec {
            channels: self.output_channels as u16,
            sample_rate: self.output_sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer =
            hound::WavWriter::create(path, spec).context("Failed to create WAV writer")?;

        for &sample in samples {
            writer
                .write_sample(sample)
                .context("Failed to write sample")?;
        }

        writer.finalize().context("Failed to finalize WAV file")?;
        Ok(())
    }
}
