
//! Feeds back the input stream directly into the output stream.
//!
//! Assumes that the input and output devices can use the same stream configuration and that they
//! support the f32 sample format.
//!
//! Uses a delay of `LATENCY_MS` milliseconds in case the default input and output streams are not
//! precisely synchronised.

extern crate anyhow;
extern crate cpal;
extern crate ringbuf;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{SharedRb};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;
use crate::utils;
use utils::make_audio_louder;

const LATENCY_MS: f32 = 7000.0;

// TODO: JPB: Add clean way to exit besides ctrl+C (which sometimes doesn't work)
// TODO: JPB: Make sure this works with other LATENCY_MS, NUM_ITERS, and NUM_ITERS_SAVED
// TODO: JPB: I think there is an issue where it doesn't compute fast enough and so it loses data


pub fn run_transcription(transcription_tx: mpsc::Sender<String>) -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    // Default devices.
    let input_device = host
        .default_input_device()
        .expect("failed to get default input device");
    println!("Using default input device: \"{}\"", input_device.name()?);

    // Top level variables
    let config: cpal::StreamConfig = input_device.default_input_config()?.into();
    let latency_frames = (LATENCY_MS / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;
    println!("{}", latency_samples);
    let sampling_freq = config.sample_rate.0 as f32 / 2.0; // TODO: JPB: Divide by 2 because of stereo to mono

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


    let whisper_path = Path::new("src/ggml-base.en.bin");
    if !whisper_path.exists() && !whisper_path.is_file() {
        panic!("expected a whisper directory")
    }
    let ctx = WhisperContext::new(&whisper_path.to_string_lossy()).expect("failed to open model");
    let mut state = ctx.create_state().expect("failed to create key");

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    println!("Setup input stream");
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        LATENCY_MS
    );
    input_stream.play()?;

    // Remove the initial samples
    consumer.pop_iter().count();

    sleep(Duration::from_millis(1000));

    loop {

        let samples: Vec<_> = consumer.pop_iter().collect();
        let samples = whisper_rs::convert_stereo_to_mono_audio(&samples).unwrap();
        let samples = make_audio_louder(&samples, 1.0);

        // let first_two_seconds = ((config.sample_rate.0 * 2000) / 1000) as usize;
        // let first_n_samples = &samples[0..2000];

        // TODO: The sampling_freq is divided by two because of stereo, need to check if this is correct for the vad
        if utils::vad_simple(&samples, sampling_freq as usize, 1000) {
            println!("Speech detected! Processing...");
            let params = gen_whisper_params();
            // params.set_tokens(&tokens);

            state
                .full(params, &*samples)
                .expect("failed to convert samples");

            let num_tokens = state.full_n_tokens(0)?;
            let words = (1..num_tokens - 1)
                .map(|i| state.full_get_token_text(0, i).expect("Error"))
                .collect::<String>();
            transcription_tx.send(words);
        }
        // Else, there is just silence. The samples should be deleted
        println!("Silence Detected!");
        sleep(Duration::from_secs(3));
    }
}

fn gen_whisper_params<'a>() -> FullParams<'a, 'a> {
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
    //params.set_n_threads(4);

    //params.set_no_speech_thold(0.3);
    //params.set_split_on_word(true);

    // This impacts token times, don't use
    //params.set_single_segment(true);

    params
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
