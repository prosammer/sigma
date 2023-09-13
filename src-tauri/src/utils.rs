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