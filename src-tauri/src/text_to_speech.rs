use std::collections::HashMap;
use std::env;
use anyhow::Result;
use async_openai::Client;
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs};
use bytes::Bytes;
use reqwest::Error;

pub async fn get_completion(messages: Vec<ChatCompletionRequestMessage>) -> Result<String> {
    let client = Client::new();

    // TODO: Use streaming and pass each word to eleven labs
    let request = match CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .max_tokens(120_u16)
        .messages(messages.clone())
        .build() {
        Ok(req) => req,
        Err(err) => {
            println!("Error building request: {}", err);
            return Err(err.into())
        }
    };

    return match client.chat().create(request).await {
        Ok(resp) => {
            match &resp.choices[0].message.content {
                Some(s) => Ok(s.to_string()),
                None => Err(anyhow::Error::msg("No content in response"))
            }
        }
        Err(err) => {
            println!("Error making completion request: {}", err);
            Err(err.into())
        }
    };
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

    return match res.error_for_status() {
        Ok(res) => {
            let audio_bytes = res.bytes().await?;
            Ok(audio_bytes)
        },
        Err(err) => {
            println!("Error: {:?}", err);
            Err(err)
        }
    }
}
