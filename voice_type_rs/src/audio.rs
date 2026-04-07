use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    stream: Option<cpal::Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 16000,
            channels: 1,
        }
    }

    pub fn start_recording(&mut self) -> anyhow::Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let config = device.default_input_config()?;
        let sample_format = config.sample_format();
        self.sample_rate = config.sample_rate().into();
        self.channels = config.channels();
        
        let stream_config: StreamConfig = config.clone().into();
        
        println!("🎤 Using audio device: {:?}", device.name().unwrap_or_else(|_| "Unknown".to_string()));
        println!("📊 Format: {:?}, Rate: {}Hz, Channels: {}", sample_format, self.sample_rate, self.channels);

        // Ensure buffer is empty
        {
            let mut buf = self.buffer.lock().unwrap();
            buf.clear();
        }

        let buffer_clone = Arc::clone(&self.buffer);
        
        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &_| {
                        let mut buf = buffer_clone.lock().unwrap();
                        buf.extend_from_slice(data);
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None
                )?
            },
            SampleFormat::I16 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &_| {
                        let mut buf = buffer_clone.lock().unwrap();
                        for &sample in data {
                            buf.push(sample.to_sample::<f32>());
                        }
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None
                )?
            },
            SampleFormat::U16 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &_| {
                        let mut buf = buffer_clone.lock().unwrap();
                        for &sample in data {
                            buf.push(sample.to_sample::<f32>());
                        }
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None
                )?
            },
            _ => return Err(anyhow::anyhow!("Unsupported sample format: {:?}", sample_format)),
        };

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.stream = None; 
        let mut buf = self.buffer.lock().unwrap();
        let raw_data = buf.clone();
        buf.clear();
        
        if raw_data.is_empty() {
            return Vec::new();
        }

        // 1. Downmix to Mono
        let mut mono_data = if self.channels > 1 {
            let mut data = Vec::with_capacity(raw_data.len() / self.channels as usize);
            for chunk in raw_data.chunks_exact(self.channels as usize) {
                let sum: f32 = chunk.iter().sum();
                data.push(sum / self.channels as f32);
            }
            data
        } else {
            raw_data
        };
        
        // 2. Simple Normalization & DC Offset removal (prevention of "audio explosions")
        let mean: f32 = mono_data.iter().sum::<f32>() / mono_data.len() as f32;
        for s in &mut mono_data {
            *s -= mean;
        }
        
        // Find peak for normalization
        let max_abs = mono_data.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if max_abs > 0.0 {
            let multiplier = 0.9 / max_abs;
            for s in &mut mono_data {
                *s *= multiplier;
            }
        }
        
        // 3. Resample to 16000 Hz using linear interpolation
        let target_rate = 16000.0;
        if (self.sample_rate as f32 - target_rate).abs() < 10.0 {
            return mono_data;
        }
        
        let ratio = self.sample_rate as f32 / target_rate;
        let mut resampled = Vec::new();
        let mut float_index = 0.0;
        
        while (float_index as usize) < mono_data.len() - 1 {
            let i = float_index as usize;
            let frac = float_index - i as f32;
            let sample = mono_data[i] * (1.0 - frac) + mono_data[i + 1] * frac;
            resampled.push(sample);
            float_index += ratio;
        }

        resampled
    }
}
