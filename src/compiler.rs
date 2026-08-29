use std::fs;
use std::path::Path;
use std::error::Error;
use std::fmt;

use crate::schema::JaffScene;
use crate::formula::TrajectoryEvaluator;
use crate::spatial::{pan_3d, NUM_CHANNELS, SPEAKER_POSITIONS};

#[derive(Debug)]
pub enum CompilerError {
    IoError(std::io::Error),
    HoundError(hound::Error),
    JsonError(serde_json::Error),
    FormulaError(crate::formula::FormulaError),
    NoObjects,
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::HoundError(e) => write!(f, "WAV audio error: {}", e),
            Self::JsonError(e) => write!(f, "JSON parse error: {}", e),
            Self::FormulaError(e) => write!(f, "Trajectory formula error: {}", e),
            Self::NoObjects => write!(f, "Scene contains no objects to compile"),
        }
    }
}

impl Error for CompilerError {}

impl From<std::io::Error> for CompilerError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<hound::Error> for CompilerError {
    fn from(e: hound::Error) -> Self {
        Self::HoundError(e)
    }
}

impl From<serde_json::Error> for CompilerError {
    fn from(e: serde_json::Error) -> Self {
        Self::JsonError(e)
    }
}

impl From<crate::formula::FormulaError> for CompilerError {
    fn from(e: crate::formula::FormulaError) -> Self {
        Self::FormulaError(e)
    }
}

struct SourceAudio {
    samples: Vec<f32>,
    sample_rate: u32,
}

/// Reads a WAV file, converts all integer/float samples to f32 [-1.0, 1.0], and downmixes to mono.
fn read_wav_to_mono_f32<P: AsRef<Path>>(path: P) -> Result<SourceAudio, hound::Error> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let num_channels = spec.channels as usize;

    let raw_samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect()
        }
        hound::SampleFormat::Int => {
            let max_val = (1_i64 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>()
                .map(|s| {
                    let val = s.unwrap_or(0);
                    val as f32 / max_val
                })
                .collect()
        }
    };

    let mut mono_samples = Vec::new();
    if num_channels == 1 {
        mono_samples = raw_samples;
    } else {
        let total_frames = raw_samples.len() / num_channels;
        mono_samples.reserve(total_frames);
        for f in 0..total_frames {
            let mut sum = 0.0;
            for c in 0..num_channels {
                sum += raw_samples[f * num_channels + c];
            }
            mono_samples.push(sum / (num_channels as f32));
        }
    }

    Ok(SourceAudio {
        samples: mono_samples,
        sample_rate: spec.sample_rate,
    })
}

/// Dynamic linear resampler that fetches a sample from a source array.
fn get_sample_resampled(samples: &[f32], original_rate: f64, target_rate: f64, local_idx_exact: f64) -> f32 {
    let src_idx = local_idx_exact * (original_rate / target_rate);
    let idx_floor = src_idx.floor() as usize;
    let idx_ceil = (idx_floor + 1).min(samples.len() - 1);
    if idx_floor >= samples.len() {
        return 0.0;
    }
    let frac = src_idx - idx_floor as f64;
    samples[idx_floor] * (1.0 - frac as f32) + samples[idx_ceil] * frac as f32
}

/// Compiles a JAFF scene file to a 7.1.4 multi-channel WAV file.
pub fn compile_scene<P1: AsRef<Path>, P2: AsRef<Path>>(
    jaff_path: P1,
    output_path: P2,
) -> Result<(), CompilerError> {
    let jaff_content = fs::read_to_string(&jaff_path)?;
    let scene = JaffScene::from_json_str(&jaff_content)?;

    if scene.objects.is_empty() {
        return Err(CompilerError::NoObjects);
    }

    let jaff_dir = jaff_path.as_ref().parent().unwrap_or_else(|| Path::new("."));

    // 1. Load all source audio files
    // To determine the output sample rate, we will use the sample rate of the first object's sound file,
    // defaulting to 48000 Hz if none can be loaded.
    let mut sources = Vec::new();
    let mut target_sample_rate = 48000;
    let mut has_sample_rate = false;

    for obj in &scene.objects {
        let sound_path = jaff_dir.join(&obj.metadata.source_sound_file);
        println!("Loading source: {:?}", sound_path);
        let audio = read_wav_to_mono_f32(&sound_path)?;
        if !has_sample_rate {
            target_sample_rate = audio.sample_rate;
            has_sample_rate = true;
        }
        sources.push(audio);
    }

    let target_rate_f = target_sample_rate as f64;

    // 2. Pre-calculate end offsets and find total output duration
    let mut compiled_objects = Vec::new();
    let mut max_end_frame = 0;

    for (i, obj) in scene.objects.iter().enumerate() {
        let audio = &sources[i];
        let original_rate_f = audio.sample_rate as f64;
        let source_len_seconds = audio.samples.len() as f64 / original_rate_f;

        let start_frame = (obj.temporal.start_offset * target_rate_f).round() as i64;
        
        let end_frame = if let Some(end_val) = obj.temporal.end_offset {
            (end_val * target_rate_f).round() as i64
        } else {
            start_frame + ((source_len_seconds * target_rate_f).round() as i64)
        };

        if end_frame > start_frame {
            if end_frame > max_end_frame {
                max_end_frame = end_frame;
            }
        }

        let evaluator = TrajectoryEvaluator::new(
            obj.spatial.start_x,
            obj.spatial.start_y,
            obj.spatial.start_z,
            obj.spatial.xformula.as_deref(),
            obj.spatial.yformula.as_deref(),
            obj.spatial.zformula.as_deref(),
            obj.spatial.volume.as_deref(),
        )?;

        compiled_objects.push((obj, evaluator, start_frame, end_frame));
    }

    if max_end_frame <= 0 {
        return Err(CompilerError::NoObjects);
    }

    let total_frames = max_end_frame as usize;
    println!("Scene Duration: {:.3}s ({} frames at {} Hz)", total_frames as f64 / target_rate_f, total_frames, target_sample_rate);

    // Allocate flat mix buffer for 12 channels
    let mut mix_buffer = vec![0.0_f32; total_frames * NUM_CHANNELS];

    // 3. Process each object frame-by-frame and mix
    for (i, (obj, evaluator, start_frame, end_frame)) in compiled_objects.iter().enumerate() {
        let audio = &sources[i];
        let original_rate_f = audio.sample_rate as f64;
        let source_len_frames = audio.samples.len();
        let source_duration_sec = source_len_frames as f64 / original_rate_f;

        let start_f = (*start_frame).max(0) as usize;
        let end_f = (*end_frame).min(max_end_frame) as usize;

        // State for dynamic proximity bass-boost filter (low-pass component)
        let mut prev_low_val = 0.0_f32;
        // 120 Hz crossover coefficient: alpha = 2 * pi * fc / fs
        let alpha = (2.0 * std::f32::consts::PI * 120.0 / (target_sample_rate as f32)).min(1.0).max(0.0);

        for f in start_f..end_f {
            let t = f as f64 / target_rate_f;
            let t_local = t - obj.temporal.start_offset;

            // Evaluate trajectory and volume
            let (x, y, z, vol) = evaluator.evaluate(t);

            // Calculate panning
            let pan_gains = pan_3d(x, y, z);

            // Calculate distance from object to listener (origin 0,0,0)
            let dist_to_listener = (x * x + y * y + z * z).sqrt();

            // Calculate delay. Assume 1 coordinate unit = 5.0 meters. Speed of sound = 343.0 m/s.
            let delay_sec = dist_to_listener * 5.0 / 343.0;
            let t_delayed = t_local - delay_sec;

            // Determine index in source file
            let src_sample_exact = if obj.temporal.loop_sound {
                let mut t_local_looped = t_delayed % source_duration_sec;
                if t_local_looped < 0.0 {
                    t_local_looped += source_duration_sec;
                }
                t_local_looped * original_rate_f
            } else {
                t_delayed * original_rate_f
            };

            let src_sample = if src_sample_exact < 0.0 || src_sample_exact >= source_len_frames as f64 {
                0.0
            } else {
                get_sample_resampled(&audio.samples, original_rate_f, target_rate_f, src_sample_exact)
            };

            // Dynamic low-shelf shelving filter for proximity bass boost
            let processed_sample = if obj.spatial.proximity_bass_boost {
                let low_val = alpha * src_sample + (1.0 - alpha) * prev_low_val;
                prev_low_val = low_val;

                // Distance relative to the TV position at (0.0, 1.0, 0.0)
                let dx = x;
                let dy = y - 1.0;
                let dz = z;
                let dist_tv = (dx * dx + dy * dy + dz * dz).sqrt();
                let bass_boost_db = if dist_tv < 0.2 {
                    (1.0 - dist_tv) * 6.0
                } else {
                    0.0
                };
                let boost_factor = 10.0_f64.powf(bass_boost_db / 20.0) as f32;

                let high_val = src_sample - low_val;
                high_val + low_val * boost_factor
            } else {
                src_sample
            };

            // Mix into output channels
            let final_gain = processed_sample * (vol as f32);
            for c in 0..NUM_CHANNELS {
                mix_buffer[f * NUM_CHANNELS + c] += final_gain * (pan_gains[c] as f32);
            }
        }
    }

    // 4. Automatic Peak Normalization (to avoid digital clipping)
    let mut peak = 0.0_f32;
    for &sample in &mix_buffer {
        let abs_val = sample.abs();
        if abs_val > peak {
            peak = abs_val;
        }
    }

    let scale = if peak > 1.0 {
        println!("Clipping detected! Peak is {:.2}. Automatically normalizing output level to 0.95.", peak);
        0.95 / peak
    } else {
        1.0
    };

    // 5. Write out multi-channel WAV file with WAVEFORMATEXTENSIBLE
    println!("Writing 7.1.4 multi-channel output to: {:?}", output_path.as_ref());
    
    let mut file = std::fs::File::create(output_path)?;
    use std::io::Write;

    let num_channels: u16 = 12;
    let bits_per_sample: u16 = 16;
    let byte_rate = target_sample_rate * (num_channels as u32) * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = (mix_buffer.len() * 2) as u32; // 2 bytes per sample (i16)
    let chunk_size = 36 + 24 + data_size; // 4 (WAVE) + 48 (fmt chunk including header) + 8 (data chunk header) + data_size - 8
    let chunk_size = 68 + data_size; // RIFF total size minus 8

    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&chunk_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&40u32.to_le_bytes())?; // Subchunk1Size (40 for extensible)
    file.write_all(&0xFFFEu16.to_le_bytes())?; // AudioFormat (WAVE_FORMAT_EXTENSIBLE)
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&target_sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    
    // Extensible specific
    file.write_all(&22u16.to_le_bytes())?; // cbSize
    file.write_all(&16u16.to_le_bytes())?; // ValidBitsPerSample
    
    // dwChannelMask for 7.1.4: FL|FR|FC|LFE|BL|BR|SL|SR|TFL|TFR|TBL|TBR = 0x2D63F
    file.write_all(&0x2D63Fu32.to_le_bytes())?; 
    
    // SubFormat: KSDATAFORMAT_SUBTYPE_PCM
    file.write_all(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71])?;

    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;

    // Write samples
    let mut out_buffer = Vec::with_capacity(mix_buffer.len() * 2);
    for sample in mix_buffer {
        let scaled_sample = sample * scale;
        let clamped = scaled_sample.clamp(-1.0, 1.0);
        let sample_i16 = (clamped * i16::MAX as f32) as i16;
        out_buffer.extend_from_slice(&sample_i16.to_le_bytes());
    }
    file.write_all(&out_buffer)?;
    file.flush()?;

    println!("Compilation successful!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_test_sine_wav<P: AsRef<Path>>(path: P, sample_rate: u32, duration_sec: f64) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let num_samples = (duration_sec * sample_rate as f64) as usize;
        for i in 0..num_samples {
            let t = i as f64 / sample_rate as f64;
            // 440 Hz Sine wave
            let sample = (t * 440.0 * 2.0 * std::f64::consts::PI).sin();
            let sample_i16 = (sample * 0.5 * i16::MAX as f64) as i16; // Half amplitude
            writer.write_sample(sample_i16).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn test_end_to_end_compilation() {
        let temp_dir = std::env::temp_dir();
        let test_src_wav = temp_dir.join("test_sine_src.wav");
        let test_jaff_path = temp_dir.join("test_scene.jaff");
        let test_output_wav = temp_dir.join("test_output_714.wav");

        // Write source WAV (1.0s of 440Hz sine wave, at 48000Hz)
        write_test_sine_wav(&test_src_wav, 48000, 1.0);

        // Write JAFF JSON scene content
        // The sound starts at 0.5 seconds and ends at 1.5 seconds.
        // It is located at (0.0, 1.0, 0.0), which maps to Bed Center.
        let jaff_json = format!(
            r#"{{
                "title": "Integration Test Scene",
                "objects": [
                    {{
                        "metadata": {{
                            "title": "Sine wave 1",
                            "source_sound_file": "{}"
                        }},
                        "temporal": {{
                            "start_offset": 0.5,
                            "end_offset": 1.5,
                            "loop": false
                        }},
                        "spatial": {{
                            "startX": 0.0,
                            "startY": 1.0,
                            "startZ": 0.0,
                            "xformula": "0.0",
                            "yformula": "0.0",
                            "zformula": "0.0",
                            "volume": "1.0"
                        }}
                    }}
                ]
            }}"#,
            test_src_wav.to_str().unwrap().replace('\\', "/")
        );

        fs::write(&test_jaff_path, jaff_json).unwrap();

        // Compile the scene
        compile_scene(&test_jaff_path, &test_output_wav).unwrap();

        // Verify compiled output file exists and matches specifications
        assert!(test_output_wav.exists());
        let mut reader = hound::WavReader::open(&test_output_wav).unwrap();
        let spec = reader.spec();

        assert_eq!(spec.channels, 12);
        assert_eq!(spec.sample_rate, 48000);
        assert_eq!(spec.bits_per_sample, 16);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);

        // Duration of output should be exactly 1.5s (72000 frames)
        let total_frames = reader.duration() as usize;
        assert_eq!(total_frames, 72000);

        // Read samples and verify temporal / spatial characteristics
        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(samples.len(), 72000 * 12);

        // First 0.5s (24000 frames) should be completely silent (all channels = 0)
        for frame in 0..24000 {
            for chan in 0..12 {
                assert_eq!(samples[frame * 12 + chan], 0);
            }
        }

        // From 0.5s to 1.5s (frames 24000 to 72000):
        // Only channel 2 (C / Center) should contain panned audio.
        // Other channels (0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11) should be completely silent.
        let mut non_zero_samples_found = false;
        for frame in 24000..72000 {
            for chan in 0..12 {
                let sample_val = samples[frame * 12 + chan];
                if chan == 2 {
                    if sample_val != 0 {
                        non_zero_samples_found = true;
                    }
                } else {
                    assert_eq!(sample_val, 0, "Expected channel {} at frame {} to be silent, but got {}", chan, frame, sample_val);
                }
            }
        }

        assert!(non_zero_samples_found, "Expected to find audio in Channel 2 (Center)");

        // Clean up temporary files
        let _ = fs::remove_file(test_src_wav);
        let _ = fs::remove_file(test_jaff_path);
        let _ = fs::remove_file(test_output_wav);
    }

    #[test]
    fn test_proximity_bass_boost_compilation() {
        let temp_dir = std::env::temp_dir();
        let test_src_wav = temp_dir.join("test_bass_src.wav");
        let test_jaff_path = temp_dir.join("test_bass_scene.jaff");
        let test_output_wav = temp_dir.join("test_bass_output_714.wav");

        // Write a 48000Hz WAV with 1.0s duration
        write_test_sine_wav(&test_src_wav, 48000, 1.0);

        // Define a scene with proximity_bass_boost enabled at the TV position (x=0, y=1, z=0)
        let jaff_json = format!(
            r#"{{
                "title": "Bass Boost Test Scene",
                "objects": [
                    {{
                        "metadata": {{
                            "title": "Close Sound",
                            "source_sound_file": "{}"
                        }},
                        "temporal": {{
                            "start_offset": 0.0,
                            "end_offset": 0.5,
                            "loop": false
                        }},
                        "spatial": {{
                            "startX": 0.0,
                            "startY": 1.0,
                            "startZ": 0.0,
                            "xformula": "0.0",
                            "yformula": "0.0",
                            "zformula": "0.0",
                            "volume": "1.0",
                            "proximity_bass_boost": true
                        }}
                    }}
                ]
            }}"#,
            test_src_wav.to_str().unwrap().replace('\\', "/")
        );

        fs::write(&test_jaff_path, jaff_json).unwrap();

        compile_scene(&test_jaff_path, &test_output_wav).unwrap();

        assert!(test_output_wav.exists());

        // Clean up
        let _ = fs::remove_file(test_src_wav);
        let _ = fs::remove_file(test_jaff_path);
        let _ = fs::remove_file(test_output_wav);
    }
}
