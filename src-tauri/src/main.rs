#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod whisper_command;
mod text_to_speech;
mod utils;

use dotenv::dotenv;
use std::{thread, time::Duration};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::sleep;

use tauri::{SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, Manager, WindowUrl, WindowBuilder, ActivationPolicy};
use chrono::{Local, NaiveTime, Timelike};
use tauri_plugin_positioner::{Position, WindowExt};
use tauri_plugin_autostart::MacosLauncher;
use tauri::Wry;
use tauri_plugin_store::with_store;
use tauri_plugin_store::{StoreCollection};
use serde_json::Value as JsonValue;
use tokio::runtime::Runtime;
use crate::text_to_speech::{get_completion, play_audio, text_to_speech};
use crate::whisper_command::run_transcription;

fn main() {
    dotenv().ok();

    let record = CustomMenuItem::new("record".to_string(), "Record");
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
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let path = home_dir.join("Movies/Video Journals/");

            if !path.exists() {
                std::fs::create_dir_all(&path).expect("Failed to create directory");
            }

            start_notification_loop(app.handle());
            Ok(())
        })
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"])))
        .plugin(tauri_plugin_store::Builder::default().build())
        // .invoke_handler(tauri::generate_handler![get_completion])
        .system_tray(tray)
        .on_system_tray_event(|app, event| {
            match event {
                tauri::SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "record" => {
                            // let window_exists = app.get_window("recording_window").is_some();
                            // if !window_exists {
                            //     let _window = create_recording_window(&app);
                            // }
                            let (transcription_tx, transcription_rx) = mpsc::channel();
                            let (talking_tx, talking_rx) = mpsc::channel();

                            thread::spawn(move || {
                                run_transcription(transcription_tx, talking_rx).unwrap();
                            });

                            thread::spawn(move || {
                                let runtime = Runtime::new().unwrap();

                                runtime.block_on(async {
                                    loop {
                                        sleep(Duration::from_millis(200));
                                        if let Ok(transcribed_words) = transcription_rx.recv() {
                                            println!("Transcribed words: {}", transcribed_words);
                                            let gpt_response = get_completion(transcribed_words).await.expect("Unable to get completion");
                                            println!("GPT Response: {}", gpt_response);
                                            let speech_audio = text_to_speech("2EiwWnXFnvU5JabPnv8n",gpt_response).await.expect("Unable to run TTS");
                                            play_audio(speech_audio);
                                            talking_tx.send(false);
                                        }
                                    }
                                });
                            });
                        }
                        "settings" => {
                            let window_exists = app.get_window("settings_window").is_some();
                            if !window_exists {
                                let _window = create_settings_window(&app);
                            }
                        }e
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



fn start_notification_loop(handle: tauri::AppHandle) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(59));

            let stores = handle.state::<StoreCollection<Wry>>();
            let path = PathBuf::from(".settings.dat");

            let handle_clone = handle.clone();
            let mut time = JsonValue::from("15:00");
            with_store(handle_clone, stores, path, |store| {
                if let Some(stored_time) = store.get("time") {
                    println!("Retrieved time from store: {}", stored_time);
                    time = stored_time.clone();
                } else {
                    println!("No time found in store");
                }
                Ok(())
            }).expect("Failed to interact with the store");

            let parsed_time = NaiveTime::parse_from_str(time.as_str().unwrap(), "%H:%M");
            let now = Local::now();


            if now.hour() == parsed_time.unwrap().hour() && now.minute() == parsed_time.unwrap().minute() {
                println!("Opening recording window");
                let window_exists = handle.get_window("recording_window").is_some();
                if !window_exists {
                    let _window = create_recording_window(&handle);
                }
            }
        }
    });
}

fn create_recording_window(handle: &tauri::AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        handle,
        "recording_window",
        WindowUrl::App("recording".into())
    )
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .inner_size(192.0,192.0)
        .build()
        .expect("Failed to create recording_window");

    new_window.move_window(Position::TopCenter).expect("Failed to center window");
    new_window
}

fn create_settings_window(handle: &tauri::AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        handle,
        "settings_window",
        WindowUrl::App("settings".into())
    )
        .build()
        .expect("Failed to create settings_window");

    new_window
}
