use std::f32::consts::PI;

/// Pitch-Synchronous Overlap-Add (PSOLA) kernel for monophonic time-stretching
pub fn psola_stretch(input: &[f32], output: &mut [f32], stretch_factor: f32, pitch_period: usize) {
    if input.is_empty() || output.is_empty() || pitch_period == 0 {
        return;
    }

    let p = pitch_period.max(16);
    let hanning_len = 2 * p;
    let mut hanning = vec![0.0f32; hanning_len];
    for i in 0..hanning_len {
        hanning[i] = 0.5 * (1.0 - (2.0 * PI * i as f32 / (hanning_len - 1) as f32).cos());
    }

    let mut out_pos = 0usize;
    let mut in_pos = 0.0f32;

    while out_pos + hanning_len <= output.len() && (in_pos as usize) + hanning_len <= input.len() {
        let in_idx = in_pos as usize;
        for i in 0..hanning_len {
            if in_idx + i < input.len() && out_pos + i < output.len() {
                output[out_pos + i] += input[in_idx + i] * hanning[i];
            }
        }
        out_pos += p;
        in_pos += (p as f32) / stretch_factor;
    }
}

/// Granular overlap-add time-stretch kernel (Tempophone & Rhythmic modes)
pub fn granular_stretch(
    input: &[f32],
    output: &mut [f32],
    stretch_factor: f32,
    grain_size: usize,
    hop_size: usize,
) {
    if input.is_empty() || output.is_empty() || grain_size == 0 || hop_size == 0 {
        return;
    }

    let grain_len = grain_size.min(input.len());
    let mut window = vec![0.0f32; grain_len];
    for i in 0..grain_len {
        window[i] = 0.5 * (1.0 - (2.0 * PI * i as f32 / (grain_len - 1) as f32).cos());
    }

    let mut out_idx = 0usize;
    let mut in_pos = 0.0f32;

    while out_idx + grain_len <= output.len() && (in_pos as usize) + grain_len <= input.len() {
        let in_idx = in_pos as usize;
        for i in 0..grain_len {
            output[out_idx + i] += input[in_idx + i] * window[i];
        }
        out_idx += hop_size;
        in_pos += (hop_size as f32) / stretch_factor;
    }
}

/// Slicing mode: stretches gaps or holds transients intact
pub fn slicing_stretch(
    input: &[f32],
    output: &mut [f32],
    transient_markers: &[usize],
    stretch_factor: f32,
) {
    if input.is_empty() || output.is_empty() {
        return;
    }

    let mut markers = transient_markers.to_vec();
    markers.sort_unstable();
    if markers.is_empty() || markers[0] != 0 {
        markers.insert(0, 0);
    }
    if *markers.last().unwrap() < input.len() {
        markers.push(input.len());
    }

    let mut out_write = 0usize;
    for window in markers.windows(2) {
        let (start, end) = (window[0], window[1]);
        let slice_len = end - start;
        let target_len = ((slice_len as f32) * stretch_factor).round() as usize;

        // Copy transient attack head as-is (first 512 samples or half slice)
        let attack_len = 512.min(slice_len / 2).max(1);
        let copy_len = attack_len.min(output.len().saturating_sub(out_write));
        for i in 0..copy_len {
            output[out_write + i] = input[start + i];
        }
        out_write += copy_len;

        // Granular fill / stretch tail of the slice
        if slice_len > attack_len && out_write < output.len() {
            let tail_in = &input[start + attack_len..end];
            let remaining_out = target_len.saturating_sub(attack_len).min(output.len() - out_write);
            granular_stretch(tail_in, &mut output[out_write..out_write + remaining_out], stretch_factor, 256, 128);
            out_write += remaining_out;
        }
    }
}
