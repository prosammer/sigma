use std::collections::HashMap;
use std::env;
use anyhow::{Error, Result};
use async_openai::Client;
use async_openai::types::{ChatCompletionFunctionsArgs, ChatCompletionRequestMessage, ChatCompletionResponseStream, CreateChatCompletionRequestArgs};
use bytes::Bytes;
use serde_json::json;

pub async fn get_gpt_response(messages: Vec<ChatCompletionRequestMessage>) -> Result<ChatCompletionResponseStream, Error> {
    let client = Client::new();

    let function = ChatCompletionFunctionsArgs::default()
        .name("leave_conversation")
        .description("The GPT AI can choose to call this function to leave the conversation whenever it appears finished, or if the user is unintelligible more than 3 times in a row.")
        .parameters(json!({"type": "object", "properties": {}}))
        .build()?;

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .max_tokens(120_u16)
        .messages(messages.clone())
        .functions(vec![function])
        .build()?;

    let stream = client.chat().create_stream(request).await?;

    return Ok(stream);
}

pub async fn text_to_speech(voice_id: &str, text: String) -> Result<Bytes, Error> {
    let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}/stream", voice_id);

    let mut data = HashMap::new();
    data.insert("text", text);

    let api_key = env::var("ELEVEN_LABS_API_KEY").expect("ELEVEN_LABS_API_KEY must be set");


    let client = reqwest::Client::new();
    let res = client.post(&url)
        .header("Accept", "audio/mpeg")
        .header("Content-Type", "application/json")
        .header("xi-api-key", api_key)
        .json(&data)
        .send()
        .await?;

    return Ok(res.bytes().await?);
}
