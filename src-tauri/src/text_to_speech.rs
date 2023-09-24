use std::collections::HashMap;
use std::env;
use anyhow::{Error, Result};
use async_openai::Client;
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role};
use bytes::Bytes;
use crate::whisper::create_chat_completion_request_msg;

pub async fn get_gpt_response(messages: Vec<ChatCompletionRequestMessage>) -> Result<ChatCompletionRequestMessage, Error> {
    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .max_tokens(120_u16)
        .messages(messages.clone())
        .build()?;

    let resp = client.chat().create(request).await?;

    let bot_string = resp.choices[0].message.content.as_ref().unwrap().clone();

    let new_bot_message = create_chat_completion_request_msg(
        bot_string,
        Role::Assistant);

    return Ok(new_bot_message);
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
