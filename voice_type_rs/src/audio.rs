use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
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

        let mut supported_configs = device.supported_input_configs()?;
        let supported_config = supported_configs
            .find(|c| c.sample_format() == SampleFormat::F32)
            .ok_or_else(|| anyhow::anyhow!("No F32 supported input config"))?;
            
        // We just use the max sample rate of the config (usually the only one like 44100 or 48000)
        let sample_rate = supported_config.max_sample_rate();
        let config = supported_config.with_sample_rate(sample_rate);
        
        self.sample_rate = sample_rate;
        self.channels = config.channels();

        let config: StreamConfig = config.into();

        // Ensure buffer is empty
        {
            let mut buf = self.buffer.lock().unwrap();
            buf.clear();
        }

        let buffer_clone = Arc::clone(&self.buffer);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| {
                let mut buf = buffer_clone.lock().unwrap();
                buf.extend_from_slice(data);
            },
            move |err| {
                eprintln!("Audio input stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.stream = None; 
        let mut buf = self.buffer.lock().unwrap();
        let raw_data = buf.clone();
        buf.clear();
        
        // 1. Downmix to Mono
        let mut mono_data = Vec::with_capacity(raw_data.len() / self.channels as usize);
        for chunk in raw_data.chunks_exact(self.channels as usize) {
            let sum: f32 = chunk.iter().sum();
            mono_data.push(sum / self.channels as f32);
        }
        
        // 2. Resample to 16000 Hz using basic linear interpolation
        let target_rate = 16000.0;
        // If we happen to be close enough, just return it
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
