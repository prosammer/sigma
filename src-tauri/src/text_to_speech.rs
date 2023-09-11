use std::collections::HashMap;
use std::env;
use std::io::Cursor;

use async_openai::{
    types::{CreateCompletionRequestArgs},
    Client,
};
use bytes::Bytes;
use reqwest::Error;
use rodio::{Decoder, OutputStream, Sink};



pub async fn get_completion(transcribed_words: String) {
    let client = Client::new();
    println!("Received: {}", transcribed_words);

    // TODO: Use streaming and pass each word to eleven labs
    let request = match CreateCompletionRequestArgs::default()
        .model("text-davinci-003")
        .prompt(format!("You are an AI personal routine trainer, please respond to this user (they communicate via speech-to-text): {}", transcribed_words))
        .max_tokens(40_u16)
        .build() {
        Ok(req) => req,
        Err(err) => {
            println!("Error building request: {}", err);
            return;
        }
    };

    let response = match client.completions().create(request).await {
        Ok(resp) => resp,
        Err(err) => {
            println!("Error making completion request: {}", err);
            return;
        }
    };

    println!("GPT Response: {}", response.choices[0].text);
    text_to_speech("21m00Tcm4TlvDq8ikWAM", &response.choices[0].text).await.expect("Unable to run TTS");
}


pub async fn text_to_speech(voice_id: &str, text: &str) -> Result<(), Error> {
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

    play_audio(audio_bytes);

    Ok(())
}

fn play_audio(audio_bytes: Bytes) {
    let cursor = Cursor::new(audio_bytes);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = Decoder::new(cursor).unwrap();
    sink.append(source);

    sink.sleep_until_end();
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use super::*;

    #[test]
    fn test_play_audio() {
        let mut file = File::open("test.wav").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        play_audio(Bytes::from(buffer));
    }
}
