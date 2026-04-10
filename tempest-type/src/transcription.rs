use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

pub struct Transcriber {
    context: WhisperContext,
}

impl Transcriber {
    pub async fn new() -> anyhow::Result<Self> {
        let model_path = Self::ensure_model_downloaded().await?;
        
        let mut params = WhisperContextParameters::default();
        params.use_gpu = true; 
        
        let model_path_str = model_path.to_string_lossy();
        let context = WhisperContext::new_with_params(model_path_str.as_ref(), params)
            .map_err(|e| anyhow::anyhow!("Failed to load whisper context: {}", e))?;
            
        Ok(Self { context })
    }

    pub fn transcribe(&mut self, audio_data: &[f32]) -> anyhow::Result<String> {
        let mut state = self.context.create_state()
            .map_err(|e| anyhow::anyhow!("Failed to create state: {}", e))?;
            
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state.full(params, audio_data)
            .map_err(|e| anyhow::anyhow!("Transcription failed: {}", e))?;

        let num_segments = state.full_n_segments();
            
        let mut res = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(text) = segment.to_str_lossy() {
                    res.push_str(&text);
                }
            }
        }
        
        Ok(res.trim().to_string())
    }

    async fn ensure_model_downloaded() -> anyhow::Result<PathBuf> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let model_dir = Path::new(&home).join(".tempest-type").join("models");
        fs::create_dir_all(&model_dir)?;
        
        // ggml-small.en.bin
        let model_path = model_dir.join("ggml-small.en.bin");
        
        if !model_path.exists() {
            println!("Downloading Whisper model (ggml-small.en.bin), ~480MB. This may take a minute...");
            let url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin";
            let response = reqwest::get(url).await?;
            let mut file = fs::File::create(&model_path)?;
            let bytes = response.bytes().await?;
            file.write_all(&bytes)?;
            println!("Download complete!");
        }
        
        Ok(model_path)
    }
}
