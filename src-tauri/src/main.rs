#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::env;
use std::{thread, time::Duration};

use async_openai::{
    types::{CreateCompletionRequestArgs},
    Client,
};

use tauri::{SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, Manager};
use chrono::{Local, Timelike};


fn main() {
    dotenv().ok();

    let hide = CustomMenuItem::new("record".to_string(), "Record new journal");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let tray = SystemTray::new().with_menu(tray_menu);

    // TODO: Add tauri autostart (on login) plugin
    tauri::Builder::default()
        .setup(|app| {
            start_3pm_event_loop(app.handle());
            Ok(())
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![get_completion])
        .system_tray(tray)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}



#[tauri::command]
async fn get_completion(name: &str) -> Result<String, String> {
    println!("get_completion called!");
    // let openai_api_key = env::var("OPENAI_API_KEY").map_err(|err| err.to_string())?;
    let client = Client::new();

    let request = CreateCompletionRequestArgs::default()
        .model("text-davinci-003")
        .prompt(format!("Write a joke about the name: {name}"))
        .max_tokens(40_u16)
        .build()
        .map_err(|err| err.to_string())?;

    let response = client.completions().create(request).await.map_err(|err| err.to_string())?;

    Ok(format!("Hello, {}!", response.choices[0].text))
}

fn start_3pm_event_loop(handle: tauri::AppHandle) {
    println!("start_3pm_event_loop called!");
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));

            // Check the current time
            let now = Local::now();
            // TODO: This is currently set to true for testing purposes
            let correct_time = true || now.hour() == 15 && now.minute() == 0;
            if correct_time {
                let _window = match handle.get_window("recording_window") {
                    Some(window) => window,
                    None => tauri::WindowBuilder::new(
                        &handle,
                        "recording_window",
                        tauri::WindowUrl::App("recording".into())
                    ).title("Recording")
                    .build()
                    .expect("Failed to create recording_window")
                };
            }
        }
    });
}