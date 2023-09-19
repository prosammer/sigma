use std::collections::HashMap;
use std::env;
use async_openai::{
    Client,
    types::CreateCompletionRequestArgs,
};
use async_openai::error::OpenAIError;
use bytes::Bytes;
use reqwest::Error;

pub async fn get_completion(transcribed_words: String) -> Result<String, OpenAIError> {
    let client = Client::new();

    // TODO: Use streaming and pass each word to eleven labs
    let request = match CreateCompletionRequestArgs::default()
        .model("text-davinci-003")
        .prompt(format!("You are an AI personal routine trainer, please respond to this user (they communicate via speech-to-text): {}", transcribed_words))
        .max_tokens(120_u16)
        .build() {
        Ok(req) => req,
        Err(err) => {
            println!("Error building request: {}", err);
            return Err(err)
        }
    };

    let response = match client.completions().create(request).await {
        Ok(resp) => resp,
        Err(err) => {
            println!("Error making completion request: {}", err);
            return Err(err)
        }
    };

    let response_text = response.choices[0].text.clone();
    return Ok(response_text);
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

    let audio_bytes = res.bytes().await?;

    Ok(audio_bytes)
}
