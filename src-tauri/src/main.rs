#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::{thread, time::Duration};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_openai::{
    types::{CreateCompletionRequestArgs},
    Client,
};

use tauri::{SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, Manager, WindowUrl, WindowBuilder, ActivationPolicy};
use chrono::{Local, NaiveTime, Timelike};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{SincFixedIn, SincInterpolationParameters, SincInterpolationType, VecResampler, WindowFunction};
use tauri_plugin_positioner::{Position, WindowExt};
use tauri_plugin_autostart::MacosLauncher;
use tauri::Wry;
use tauri_plugin_store::with_store;
use tauri_plugin_store::{StoreCollection};
use serde_json::Value as JsonValue;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};


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
        .invoke_handler(tauri::generate_handler![get_completion])
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
                            start_recording();
                        }
                        "settings" => {
                            let window_exists = app.get_window("settings_window").is_some();
                            if !window_exists {
                                let _window = create_settings_window(&app);
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

fn start_notification_loop(handle: tauri::AppHandle) {
    println!("Started event loop");

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

fn start_recording() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("Failed to get default input device");
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(48000),
        buffer_size: cpal::BufferSize::Default,
    };
    println!("Default input config: {:?}", config);

    let sample_rate = config.sample_rate.0;
    let channels = config.channels as usize;

    let audio_buffer = Arc::new(Mutex::new(Vec::new()));

    let stream = device.build_input_stream(
        &config.into(),
        {
            let audio_buffer = audio_buffer.clone();
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = audio_buffer.lock().unwrap();
                buffer.extend_from_slice(data);
            }
        },
        |err| {
            eprintln!("An error occurred on stream: {}", err);
        },
        None
    ).expect("Failed to build input stream");

    stream.play().expect("Failed to start stream");

    // For this example, let's record for 5 seconds and then transcribe
    thread::sleep(Duration::from_secs(25));
    stream.pause().expect("Failed to pause stream");

    let locked_buffer = audio_buffer.lock().unwrap();
    transcribe_audio(&locked_buffer, sample_rate, channels);
}

fn transcribe_audio(audio_buffer: &[f32], sample_rate: u32, channels: usize) {
    // Load a context and model.
    let ctx = WhisperContext::new("/Users/samfinton/Documents/Programming/tauri_sigma/src-tauri/src/ggml-base.en.bin")
        .expect("failed to load model");
    let mut state = ctx.create_state().expect("failed to create key");

    // Create a params object for running the model.
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });

    // Edit params as needed.
    params.set_n_threads(1);
    params.set_translate(true);
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // Convert the audio to floating point samples.
    if channels != 1 {
        panic!(">1 channels unsupported");
    }

    // Check if resampling is needed
    let mut audio = audio_buffer.to_vec();
    if sample_rate != 16000 {
        // Create a resampler
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        let mut resampler = SincFixedIn::<f32>::new(
            16000 as f64 / sample_rate as f64,
            2.0,
            params,
            audio_buffer.len(),
            channels,
        ).unwrap();

        // Process the audio buffer with the resampler
        let waves_in = vec![audio; channels];
        let resampled_waves = resampler.process(&waves_in, None).unwrap();

        // Flatten the resampled waves into a single audio buffer
        audio = resampled_waves.concat();
    }

    // Run the model.
    state.full(params, &audio[..]).expect("failed to run model");

    // Iterate through the segments of the transcript.
    let num_segments = state
        .full_n_segments()
        .expect("failed to get number of segments");
    for i in 0..num_segments {
        // Get the transcribed text and timestamps for the current segment.
        let segment = state
            .full_get_segment_text(i)
            .expect("failed to get segment");
        let start_timestamp = state
            .full_get_segment_t0(i)
            .expect("failed to get start timestamp");
        let end_timestamp = state
            .full_get_segment_t1(i)
            .expect("failed to get end timestamp");

        // Print the segment to stdout.
        println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
    }
}