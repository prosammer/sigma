#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod whisper;
mod text_to_speech;
mod stores;
mod audio_utils;

use dotenv::dotenv;
use std::{env, thread, time::Duration};
use std::sync::{Arc, mpsc, Mutex};
use std::thread::sleep;
use async_openai::types::{ChatCompletionRequestMessageArgs, Role};

use tauri::{ActivationPolicy, AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem, WindowBuilder, WindowUrl};
use chrono::{Local, NaiveTime, Timelike};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_positioner::{Position, WindowExt};
use tokio::runtime::Runtime;
use audio_utils::play_audio_bytes;
use crate::text_to_speech::{get_completion, text_to_speech};
use crate::stores::get_from_store;
use crate::whisper::run_transcription;

fn main() {
    dotenv().ok();

    let record = CustomMenuItem::new("talk".to_string(), "Talk");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(record)
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let tray = SystemTray::new().with_menu(tray_menu);

    let mut app = tauri::Builder::default()
        .setup(|app| {
            // If we're in production, start waiting for the right time to start the voice chat.
            // In dev, start it right away.
            let is_production = env::var("IS_PRODUCTION").map_or(false, |v| v == "true");
            if is_production {
                start_notification_loop(app.handle());
            } else {
                let window_exists = app.handle().get_window("transcription_window").is_some();
                if !window_exists {
                    let _window = create_transcription_window(&app.handle());
                }
            }

            Ok(())
        })
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"])))
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![start_voice_chat])
        .system_tray(tray)
        .on_system_tray_event(|app_handle, event| {
            match event {
                tauri::SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "talk" => {
                            create_transcription_window(app_handle);
                        }
                        "settings" => {
                            let window_exists = app_handle.get_window("settings_window").is_some();
                            if !window_exists {
                                let _window = create_settings_window(&app_handle);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.set_activation_policy(ActivationPolicy::Accessory);

    app.run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}

#[tauri::command]
async fn start_voice_chat(handle: AppHandle) {
    let user_first_name = get_from_store(handle.clone(),  "userFirstName");
    let initial_speech = match user_first_name {
        Some(s) => format!("Good morning {}!", s),
        None => "Good morning!".to_string(),
    };
    let initial_speech_audio = text_to_speech("pMsXgVXv3BLzUgSXRplE",initial_speech).await.expect("Unable to run TTS");
    play_audio_bytes(initial_speech_audio);

    let (transcription_tx, transcription_rx) = mpsc::channel();
    let (talking_tx, talking_rx) = mpsc::channel();

    thread::spawn(move || {
        run_transcription(transcription_tx, talking_rx).unwrap();
    });

    let messages = Arc::new(Mutex::new(Vec::new()));

    let user_prompt = get_from_store(handle.clone(), "userPrompt").unwrap_or("".to_string());

    let system_messages = [
        ChatCompletionRequestMessageArgs::default()
        .content("You are an AI personal routine trainer. You greet the user in the morning, then go through the user-provided morning routine checklist and ensure that the user completes each task on the list in order. Make sure to keep your tone positive, but it is vital that the user completes each task - do not allow them to 'skip' tasks. The user uses speech-to-text to communicate, so some of their messages may be incorrect - if some text seems out of place, please ignore it. If the users sentence makes no sense in the context, tell them you don't understand and ask them to repeat themselves. If you receive any text like [SILENCE] or [MUSIC] please respond with - I didn't catch that. The following message is the prompt the user provided - their morning checklist. ")
        .role(Role::System)
        .build()
        .unwrap(),
        ChatCompletionRequestMessageArgs::default()
        .content(user_prompt)
        .role(Role::System)
        .build()
        .unwrap()];

    messages.lock().unwrap().extend_from_slice(&system_messages);


    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();

        runtime.block_on(async {
            loop {
                if let Ok(user_sentence) = transcription_rx.recv() {
                    let mut messages = messages.lock().unwrap();

                    let user_message = ChatCompletionRequestMessageArgs::default()
                        .content(&user_sentence)
                        .role(Role::User)
                        .build()
                        .unwrap();
                    messages.push(user_message);


                    let gpt_response = get_completion(messages.clone()).await.expect("Unable to get completion");

                    let bot_message = ChatCompletionRequestMessageArgs::default()
                        .content(&gpt_response)
                        .role(Role::Assistant)
                        .build()
                        .unwrap();
                    messages.push(bot_message);

                    for message in messages.iter() {
                        println!("------------------");
                        println!("{}", message.content.as_ref().unwrap());
                        println!("------------------");
                    }


                    let speech_audio = text_to_speech("pMsXgVXv3BLzUgSXRplE",gpt_response).await.expect("Unable to run TTS");
                    play_audio_bytes(speech_audio);
                    let _send = talking_tx.send(false);
                }
            }
        });
    });
}

fn start_notification_loop(handle: AppHandle) {
    thread::spawn(move || {
        loop {
            sleep(Duration::from_secs(59));

            let time = get_from_store(handle.clone(), "time").unwrap_or("15:00".to_string());
            let parsed_time = NaiveTime::parse_from_str(&time, "%H:%M");
            let now = Local::now();

            if now.hour() == parsed_time.unwrap().hour() && now.minute() == parsed_time.unwrap().minute() {
                println!("Chosen time reached! Starting voice chat");
                create_transcription_window(&handle);
            }
        }
    });
}

fn create_transcription_window(handle: &AppHandle) -> tauri::Window {

    let window_exists = handle.get_window("transcription_window").is_some();

    if window_exists {
        return handle.get_window("transcription_window").unwrap();
    }

    let new_window = WindowBuilder::new(
        handle,
        "transcription_window",
        WindowUrl::App("transcription".into())
    )
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .inner_size(192.0,192.0)
        .build()
        .expect("Failed to create transcription_window");

    new_window.move_window(Position::TopCenter).expect("Failed to center window");
    new_window
}

fn create_settings_window(handle: &AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        handle,
        "settings_window",
        WindowUrl::App("settings".into())
    )
        .build()
        .expect("Failed to create settings_window");

    new_window
}
