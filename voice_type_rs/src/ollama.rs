use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn cleanup_text(raw_text: &str) -> anyhow::Result<String> {
    let prompt = format!(
        "Clean up this voice transcription. Fix grammar and punctuation.\n\
        - Convert \"comma\" and \"period\" to punctuation\n\
        - Fix speech-to-text errors\n\
        - Output ONLY the cleaned text\n\n\
        Raw: {}\n\n\
        Clean:",
        raw_text
    );

    let client = reqwest::Client::new();
    let req_body = OllamaRequest {
        model: "qwen2.5-coder:7b",
        prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.1,
            num_predict: 500,
        },
    };

    let res = client
        .post("http://localhost:11434/api/generate")
        .json(&req_body)
        .send()
        .await?;

    if res.status().is_success() {
        let parsed: OllamaResponse = res.json().await?;
        Ok(parsed.response.trim().to_string())
    } else {
        anyhow::bail!("Ollama error: {}", res.status())
    }
}

pub async fn summarize_memo(raw_text: &str) -> anyhow::Result<String> {
    let prompt = format!(
        "Summarize this meeting transcript into clear bullet points and action items.\n\
        Transcript:\n\
        {}\n\n\
        Summary:",
        raw_text
    );

    let client = reqwest::Client::new();
    let req_body = OllamaRequest {
        model: "qwen2.5-coder:7b",
        prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.2,
            num_predict: 1000,
        },
    };

    let res = client
        .post("http://localhost:11434/api/generate")
        .json(&req_body)
        .send()
        .await?;

    if res.status().is_success() {
        let parsed: OllamaResponse = res.json().await?;
        Ok(parsed.response.trim().to_string())
    } else {
        anyhow::bail!("Ollama error: {}", res.status())
    }
}
