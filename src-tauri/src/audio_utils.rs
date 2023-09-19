use std::error::Error;
use rodio::{Decoder, OutputStream, Sink};
use bytes::Bytes;
use std::io::Cursor;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(max).max(min)
}

pub fn make_audio_louder(audio_samples: &[f32], gain: f32) -> Vec<f32> {
    audio_samples
        .iter()
        .map(|sample| {
            let louder_sample = sample * gain;
            clamp(louder_sample, -1.0, 1.0)
        })
        .collect()
}

fn high_pass_filter(data: &mut Vec<f32>, cutoff: f32, sample_rate: f32) {
    const M_PI: f32 = std::f32::consts::PI;

    let rc = 1.0 / (2.0 * M_PI * cutoff);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    let mut y = data[0];

    for i in 1..data.len() {
        y = alpha * (y + data[i] - data[i - 1]);
        data[i] = y;
    }
}

pub(crate) fn vad_simple(
    mut pcmf32: &mut Vec<f32>,
    sample_rate: usize,
    last_ms: usize
) -> bool {
    let vad_thold = 0.6;
    let freq_thold = 100.0;

    let verbose = false;
    let n_samples = pcmf32.len();
    let n_samples_last = (sample_rate * last_ms) / 1000;

    if n_samples_last >= n_samples {
        // not enough samples - assume no speech
        return false;
    }

    if freq_thold > 0.0 {
        high_pass_filter(&mut pcmf32, freq_thold, sample_rate as f32);
    }

    let mut energy_all = 0.0f32;
    let mut energy_last = 0.0f32;

    for i in 0..n_samples {
        energy_all += pcmf32[i].clone().abs();

        if i >= n_samples - n_samples_last {
            energy_last += pcmf32[i].abs();
        }
    }

    energy_all /= n_samples as f32;
    energy_last /= n_samples_last as f32;

    if verbose {
        eprintln!(
            "vad_simple: energy_all: {}, energy_last: {}, vad_thold: {}, freq_thold: {}",
            energy_all, energy_last, vad_thold, freq_thold
        );
    }

    if energy_last > vad_thold * energy_all {
        return false;
    }

    true
}

pub fn convert_stereo_to_mono_audio(samples: Vec<&mut f32>) -> Result<Vec<f32>, &'static str> {
    if samples.len() & 1 != 0 {
        return Err("The stereo audio vector has an odd number of samples. \
            This means a half-sample is missing somewhere");
    }

    Ok(samples
        .chunks_exact(2)
        .map(|x| (*x[0] + *x[1]) / 2.0)
        .collect())
}

pub fn play_audio_bytes(audio_bytes: Bytes) {
    let cursor = Cursor::new(audio_bytes);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = Decoder::new(cursor).unwrap();
    sink.append(source);

    sink.sleep_until_end();
}

pub fn play_audio_f32_vec(data: Vec<f32>, sample_rate: u32) {
    println!("Playing audio");
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, data);
    sink.append(source);

    sink.sleep_until_end();
}

pub fn resample_audio(input: Vec<f32>, from_rate: usize, to_rate: usize) -> Result<Vec<f32>, Box<dyn Error>> {
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        oversampling_factor: 128,
        interpolation: SincInterpolationType::Linear,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = SincFixedIn::<f32>::new(
        to_rate as f64 / from_rate as f64,
        10.0,
        params,
        1,
        1,
    ).unwrap();

    let output = resampler.process(&[input], None).unwrap();

    Ok(output[0].clone()) // since there is only one channel, we return the first (and only) inner vector
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use bytes::Bytes;
    use crate::audio_utils::play_audio_bytes;

    #[test]
    fn test_play_audio() {
        let mut file = File::open("test.wav").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        play_audio_bytes(Bytes::from(buffer));
    }
}
