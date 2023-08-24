use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{SincFixedIn, SincInterpolationParameters, SincInterpolationType, VecResampler, WindowFunction};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

const CHUNK_DURATION: u64 = 10;

pub struct Transcriber<'a> {
    ctx: WhisperContext,
    state: Option<WhisperState<'a>>,
}

impl<'a> Transcriber<'a> {
    pub fn initialize(&mut self) {
        let state = self.ctx.create_state().expect("failed to create key");
        self.state = Some(state);
    }

    pub fn new(model_path: &str) -> Self {
        let ctx = WhisperContext::new(model_path).expect("Failed to load model");
        Transcriber { ctx, state: None }
    }

    pub fn transcribe(&mut self, audio_buffer: &[f32], sample_rate: u32, channels: usize) {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });

        params.set_n_threads(1);
        params.set_translate(true);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(true);
        params.set_print_timestamps(false);

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
                16000f64 / sample_rate as f64,
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

        let state = &mut self.state.as_mut().unwrap();

        // Run the model.
        state.full(params, &audio[..]).expect("failed to run model");

        // Iterate through the segments of the transcript.
        let num_segments = state.full_n_segments().expect("failed to get number of segments");

        for i in 0..num_segments {
            // Get the transcribed text and timestamps for the current segment.
            let segment = state.full_get_segment_text(i).expect("failed to get segment");
            let start_timestamp = state.full_get_segment_t0(i).expect("failed to get start timestamp");
            let end_timestamp = state.full_get_segment_t1(i).expect("failed to get end timestamp");

            // Print the segment to stdout.
            println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
        }
    }
}

pub fn start_recording(transcriber: &mut Transcriber) {
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
        None,
    ).expect("Failed to build input stream");

    stream.play().expect("Failed to start stream");

    // Instead of sleeping for 10 seconds, we'll loop and process in chunks
    loop {
        // Sleep for the chunk duration
        thread::sleep(Duration::from_secs(CHUNK_DURATION));

        let locked_buffer = audio_buffer.lock().unwrap().clone();
        transcriber.transcribe(&locked_buffer, sample_rate, channels);

        // Clear the buffer after processing
        audio_buffer.lock().unwrap().clear();
    }
}