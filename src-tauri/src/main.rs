#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::env;
use std::{thread, time::Duration};

use async_openai::{
    types::{CreateCompletionRequestArgs},
    Client,
};

use tauri::{SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, Manager, WindowUrl, WindowBuilder};
use chrono::{Local, Timelike};
use tauri_plugin_positioner::{Position, WindowExt};


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
        .plugin(tauri_plugin_positioner::init())
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

            let now = Local::now();

            // TODO: This is for testing purposes only. Normally, we would want to check for 3pm
            if true || now.hour() == 15 && now.minute() == 0 {
                let window_exists = handle.get_window("recording_window").is_some();
                if !window_exists {
                    let _window = create_and_position_window(&handle);
                }
            }
        }
    });
}

fn create_and_position_window(handle: &tauri::AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        handle,
        "recording_window",
        WindowUrl::App("recording".into())
    )
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .inner_size(400.0, 400.0)
        .build()
        .expect("Failed to create recording_window");

    new_window.move_window(Position::TopCenter).expect("Failed to center window");
    new_window
}