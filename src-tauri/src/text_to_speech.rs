use std::collections::HashMap;
use std::env;
use anyhow::{Error, Result};
use bytes::Bytes;
use tts::*;

#[cfg(target_os = "macos")]
use cocoa_foundation::base::id;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::NSDefaultRunLoopMode;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::NSRunLoop;
#[cfg(target_os = "macos")]
use objc::class;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use std::{thread, time};
use tts::*;
use tauri::AppHandle;
use crate::stores::get_from_store;

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

pub fn speak_string(text: &str) -> Result<(), Error> {
    let mut tts = Tts::default()?;
    tts.speak(text, false)?;
    #[cfg(target_os = "macos")]
    {
        let run_loop: id = unsafe { NSRunLoop::currentRunLoop() };
        unsafe {
            let date: id = msg_send![class!(NSDate), distantFuture];
            let _: () = msg_send![run_loop, runMode:NSDefaultRunLoopMode beforeDate:date];
        }
    }
    let time = time::Duration::from_secs(5);
    thread::sleep(time);
    Ok(())
}

pub async fn initial_speech(handle: AppHandle) {
    println!("Starting initial_speech");
    let user_first_name = get_from_store(handle, "userFirstName");
    let initial_speech = match user_first_name {
        Some(s) => format!("Good morning {}!", s),
        None => "Good morning!".to_string(),
    };
    speak_string(&initial_speech);
    println!("Finished initial_speech");
    // let initial_speech_audio = text_to_speech("pMsXgVXv3BLzUgSXRplE", initial_speech).await.expect("Unable to run TTS");
    // play_audio_bytes(initial_speech_audio);
}
