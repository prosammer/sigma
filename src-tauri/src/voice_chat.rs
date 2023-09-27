use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use async_openai::types::Role;
use tauri::AppHandle;
use tokio::sync::Mutex;
use crate::{text_to_speech, whisper};
use crate::audio_utils::play_audio_from_wav;
use crate::text_to_speech::speak_string;
use crate::gpt::{create_chat_completion_request_msg, get_gpt_response};
use crate::whisper::WHISPER_CONTEXT;

#[tauri::command]
pub async fn start_voice_chat(handle: AppHandle) {
    let handle_clone = handle.clone();
    let initial_speech_handle = tokio::spawn(async { text_to_speech::initial_speech(handle_clone).await });

    let (audio_tx, mut audio_rx) = tauri::async_runtime::channel(20);
    let (user_string_tx, mut user_string_rx) = tauri::async_runtime::channel(20);
    let (gpt_string_tx, mut gpt_string_rx) = tauri::async_runtime::channel(20);
    let (resume_stream_tx, resume_stream_rx) = tauri::async_runtime::channel(1);
    let should_quit = Arc::new(AtomicBool::new(false));

    let initial_messages = whisper::messages_setup(handle.clone()).await;
    let messages = Arc::new(Mutex::new(initial_messages));
    let messages_clone = messages.clone();


    whisper::init_whisper_context().await;
    let ctx = WHISPER_CONTEXT.get().expect("WhisperContext not initialized");
    let mut state = ctx.create_state().expect("failed to create key");

    initial_speech_handle.await.unwrap();

    let should_quit_clone = should_quit.clone();
    // Start the thread that sends audio to the channel
    thread::spawn(|| {
        whisper::send_system_audio_to_channel(audio_tx, resume_stream_rx, should_quit_clone);
    });

    let should_quit_clone = should_quit.clone();
    // Start the thread that takes audio from the channel and sends it to STT
    let _ = tauri::async_runtime::spawn(async move {
        loop {
            if let Some(audio) = audio_rx.recv().await {
                let text = whisper::speech_to_text(&audio, &mut state);
                println!("User: {}", text.clone());

                let new_message = create_chat_completion_request_msg(text.clone(), Role::User);
                messages_clone.lock().await.push(new_message);
                user_string_tx.send(text.clone()).await.expect("Failed to send text to channel");
            }
            if should_quit_clone.load(Relaxed) {
                break;
            }
        }
    });


    let should_quit_clone = should_quit.clone();
    // Start the thread that takes the STT response and sends it to GPT
    let _ = tauri::async_runtime::spawn(async move {
        loop {
            if let Some(_user_string) = user_string_rx.recv().await {
                let messages_clone = messages.lock().await.clone();

                let new_bot_message = get_gpt_response(messages_clone).await.expect("Failed to get GPT response");

                if new_bot_message.role == Role::System {
                    println!("Sending quit signal");
                    should_quit_clone.store(true, Relaxed);
                    play_audio_from_wav(PathBuf::from("assets/audio/session_complete.wav"));
                    break;
                }

                println!("Bot: {}", new_bot_message.content.as_ref().unwrap());
                messages.lock().await.push(new_bot_message.clone());

                gpt_string_tx.send(new_bot_message.content.unwrap()).await.expect("Failed to send message to channel");
            }
        }
    });

    let should_quit_clone = should_quit.clone();
    // Start the thread that takes the GPT response and sends it to TTS
    let _ = tauri::async_runtime::spawn(async move {
        loop {
            if let Some(gpt_response) = gpt_string_rx.recv().await {
                speak_string(gpt_response).expect("Failed to speak string");
                // let bot_message_audio = text_to_speech("pMsXgVXv3BLzUgSXRplE", gpt_response).await.expect("Unable to run TTS");
                // play_audio_bytes(bot_message_audio);
                resume_stream_tx.send(true).await.expect("Failed to send pause_stream message");
            }
            if should_quit_clone.load(Relaxed) {
                break;
            }
        }
    });

    // save the messages to

}

