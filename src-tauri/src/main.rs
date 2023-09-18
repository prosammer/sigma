#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod whisper_command;
mod text_to_speech;
mod stores;
mod audio_utils;

use dotenv::dotenv;
use std::{env, thread, time::Duration};
use std::sync::mpsc;
use std::thread::sleep;

use tauri::{SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, Manager, WindowUrl, WindowBuilder, ActivationPolicy, AppHandle};
use chrono::{Local, NaiveTime, Timelike};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_positioner::{Position, WindowExt};
use tokio::runtime::Runtime;
use crate::text_to_speech::{get_completion, play_audio, text_to_speech};
use crate::stores::get_from_store;
use crate::whisper_command::run_transcription;

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
    let user_first_name = get_from_store(handle,  "userFirstName");
    let initial_speech = match user_first_name {
        Some(s) => format!("Good morning {}!", s),
        None => "Good morning!".to_string(),
    };
    let initial_speech_audio = text_to_speech("2EiwWnXFnvU5JabPnv8n",initial_speech).await.expect("Unable to run TTS");
    play_audio(initial_speech_audio);

    let (transcription_tx, transcription_rx) = mpsc::channel();
    let (talking_tx, talking_rx) = mpsc::channel();

    thread::spawn(move || {
        run_transcription(transcription_tx, talking_rx).unwrap();
    });

    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();

        runtime.block_on(async {
            loop {
                if let Ok(transcribed_words) = transcription_rx.recv() {
                    println!("Transcribed words: {}", transcribed_words);
                    let gpt_response = get_completion(transcribed_words).await.expect("Unable to get completion");
                    println!("GPT Response: {}", gpt_response);
                    let speech_audio = text_to_speech("2EiwWnXFnvU5JabPnv8n",gpt_response).await.expect("Unable to run TTS");
                    play_audio(speech_audio);
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
