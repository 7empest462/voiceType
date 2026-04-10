use ollama_rs::Ollama;
use ollama_rs::generation::chat::{ChatMessage, request::ChatMessageRequest};
use ollama_rs::models::ModelOptions;

pub async fn cleanup_text(raw_text: &str, model: &str) -> anyhow::Result<String> {
    let system_msg = "You are an ultra-conservative transcription-correction assistant.\n\n\
                      CRITICAL RULES:\n\
                      1. DO NOT REWRITE. RETAIN every single word from the raw input.\n\
                      2. DO NOT change vocabulary, word order, or tone.\n\
                      3. DO NOT expand, summarize, or explain.\n\
                      4. Your ONLY task is to fix obvious speech-to-text spelling errors and add punctuation.\n\
                      5. If the input is clear, return it EXACTLY as it is.\n\
                      6. If the input is empty or nonsensical noise, return an EMPTY STRING.\n\
                      7. NEVER use 'Thinking' blocks or meta-comments.";
    
    let user_msg = format!("Correct spelling only, keep all words: {}", raw_text);

    let ollama = Ollama::default();
    
    let messages = vec![
        ChatMessage::system(system_msg.to_string()),
        ChatMessage::user(user_msg),
    ];

    let options = ModelOptions::default()
        .temperature(0.1)
        .num_predict(500);

    let request = ChatMessageRequest::new(model.to_string(), messages)
        .options(options);

    let res = ollama.send_chat_messages(request).await
        .map_err(|e| anyhow::anyhow!("Ollama error: {}", e))?;

    Ok(res.message.content.trim().to_string())
}

pub async fn summarize_memo(raw_text: &str, model: &str) -> anyhow::Result<String> {
    let system_msg = "You are a professional assistant. Summarize meeting transcripts into clear bullet points and action items.\n\
                      Output ONLY the summary. Do not use 'Thinking' blocks.";
    
    let user_msg = format!("Summarize this: {}", raw_text);

    let ollama = Ollama::default();

    let messages = vec![
        ChatMessage::system(system_msg.to_string()),
        ChatMessage::user(user_msg),
    ];

    let options = ModelOptions::default()
        .temperature(0.2)
        .num_predict(1000);

    let request = ChatMessageRequest::new(model.to_string(), messages)
        .options(options);

    let res = ollama.send_chat_messages(request).await
        .map_err(|e| anyhow::anyhow!("Ollama error: {}", e))?;

    Ok(res.message.content.trim().to_string())
}
