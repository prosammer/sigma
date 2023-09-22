extern crate anyhow;
extern crate cpal;
extern crate ringbuf;

use std::mem::MaybeUninit;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{Consumer, SharedRb};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use std::sync::{Arc};
use std::thread::sleep;
use std::time::Duration;
use anyhow::Error;
use async_openai::types::{ChatCompletionRequestMessageArgs, Role};
use cpal::{Stream, StreamConfig};
use tauri::AppHandle;
use tokio::runtime::Runtime;
use crate::audio_utils;
use crate::audio_utils::{convert_stereo_to_mono_audio, make_audio_louder, play_audio_bytes};
use crate::stores::get_from_store;
use crate::text_to_speech::{get_completion, text_to_speech};

const LATENCY_MS: f32 = 7000.0;

pub fn run_transcription(handle: AppHandle) -> Result<(), Error> {
    let whisper_path = Path::new("src/ggml-base.en.bin");
    if !whisper_path.exists() && !whisper_path.is_file() {
        panic!("expected a whisper directory")
    }
    let ctx = WhisperContext::new(&whisper_path.to_string_lossy()).expect("failed to open model");
    let mut state = ctx.create_state().expect("failed to create key");

    let (config, mut consumer, input_stream) = setup_audio()?;

    let mut messages = Vec::new();

    let user_prompt = get_from_store(handle, "userPrompt").unwrap_or("".to_string());

    let system_messages = [
        ChatCompletionRequestMessageArgs::default()
            .content("You are an AI personal routine trainer. You greet the user in the morning, then go through the user-provided morning routine checklist and ensure that the user completes each task on the list in order. Make sure to keep your tone positive, but it is vital that the user completes each task - do not allow them to 'skip' tasks. The user uses speech-to-text to communicate, so some of their messages may be incorrect - if some text seems out of place, please ignore it. If the users sentence makes no sense in the context, tell them you don't understand and ask them to repeat themselves. If you receive any text like [SILENCE] or [MUSIC] please respond with - I didn't catch that. The following message is the prompt the user provided - their morning checklist. Call the leave_conversation function when the user has completed their morning routine.")
            .role(Role::System)
            .build()
            .unwrap(),
        ChatCompletionRequestMessageArgs::default()
            .content(user_prompt)
            .role(Role::System)
            .build()
            .unwrap()
    ];

    messages.extend_from_slice(&system_messages);


    println!("Starting the input stream with `{}` milliseconds of latency.", LATENCY_MS);
    input_stream.play()?;
    // Remove the initial samples
    consumer.clear();
    sleep(Duration::from_millis(2000));

    let rt = Runtime::new().unwrap();

    loop {

        let samples: Vec<f32> = consumer.iter().map(|x| *x).collect();
        // TODO: Instead of removing every second sample, just set the input data fn to only push every second sample
        let samples = convert_stereo_to_mono_audio(samples).unwrap();
        let mut samples = make_audio_louder(&samples, 2.0);

        let sampling_freq = config.sample_rate.0 as f32 / 2.0; // TODO: Divide by 2 because of stereo to mono

        if audio_utils::vad_simple(&mut samples, sampling_freq as usize, 1000) {
            // the last 1000ms of audio was silent and there was talking before it
            println!("Speech detected! Pausing input stream...");
            input_stream.pause().expect("Failed to pause input stream");

            let user_sentence = speech_to_text(&samples, &mut state);

            let user_message = ChatCompletionRequestMessageArgs::default()
                .content(&user_sentence)
                .role(Role::User)
                .build()
                .unwrap();
            messages.push(user_message);

            let gpt_response = rt.block_on(get_completion(messages.clone())).expect("Unable to get completion");


            let bot_message = ChatCompletionRequestMessageArgs::default()
                .content(&gpt_response)
                .role(Role::Assistant)
                .build()
                .unwrap();
            messages.push(bot_message);

            println!("------------------");
            for message in messages.iter() {
                println!("{}: {}", message.role.to_string(), message.content.as_ref().unwrap());
                println!("------------------");
            }


            let speech_audio = rt.block_on(text_to_speech("pMsXgVXv3BLzUgSXRplE", gpt_response)).expect("Unable to run TTS");
            play_audio_bytes(speech_audio);

            consumer.clear();
            input_stream.play().expect("Failed to play input stream");
            println!("Received signal");
        } else {
            // Else, there is just silence. The samples should be deleted
            println!("Silence Detected!");
            sleep(Duration::from_secs(1));
        }
        // TODO: Clear some of the buffer to avoid latency issues - this doesn't work
        // if consumer.len() > latency_samples / 2 {
        //     println!("Clearing half of the buffer");
        //     consumer.skip(latency_samples / 2);
        // }
    }
}
fn setup_audio() -> Result<(StreamConfig, Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>, Stream), Error> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("failed to get default input device");
    println!("Using default input device: \"{}\"", input_device.name()?);
    let config = input_device
        .default_input_config()
        .expect("Failed to get default input config").config();
    println!("Default input config: {:?}", config);

    // Top level variables
    let latency_frames = (LATENCY_MS / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;
    println!("{}", latency_samples);

    // The buffer to share samples
    let ring = SharedRb::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    // Setup microphone callback
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if producer.push(sample).is_err() {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    println!("Setup input stream");
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
    Ok((config, consumer, input_stream))
}

fn speech_to_text(samples: &Vec<f32>, state: &mut WhisperState) -> String {
    let mut params = FullParams::new(SamplingStrategy::default());
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_language(Some("en"));
    params.set_token_timestamps(true);
    params.set_duration_ms(LATENCY_MS as i32);
    params.set_no_context(true);
    params.set_n_threads(8);

    //params.set_no_speech_thold(0.3);
    //params.set_split_on_word(true);

    state
        .full(params, &*samples)
        .expect("failed to convert samples");

    let num_tokens = state.full_n_tokens(0).expect("Error");
    let words = (1..num_tokens - 1)
        .map(|i| state.full_get_token_text(0, i).expect("Error"))
        .collect::<String>();

    words
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
