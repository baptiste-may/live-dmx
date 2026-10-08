use crate::output::types::GroupEffectState;

/// High-quality 64-bit mixer (SplitMix64) providing full bit avalanche.
/// Every bit of input influences every bit of output with 50% probability,
/// guaranteeing that adjacent fixture IDs and pixel indices have zero correlation.
pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Checks if a sequential run in steps is excessively long.
fn max_sequential_run(steps: &[usize]) -> usize {
    let mut max_run = 1;
    let mut current_asc = 1;
    let mut current_desc = 1;

    for w in steps.windows(2) {
        if w[1] as isize == w[0] as isize + 1 {
            current_asc += 1;
            max_run = max_run.max(current_asc);
        } else {
            current_asc = 1;
        }

        if w[1] as isize == w[0] as isize - 1 {
            current_desc += 1;
            max_run = max_run.max(current_desc);
        } else {
            current_desc = 1;
        }
    }

    max_run
}

/// Determines whether a permutation is too structured (accidentally resembles
/// a linear left chase, a right chase, or a symmetrical wing/center distribution).
pub fn is_too_structured(steps: &[usize]) -> bool {
    let count = steps.len();
    if count < 3 {
        return false;
    }

    // 1. Strictly monotonic check (Left and Right)
    let is_strictly_ascending = steps.windows(2).all(|w| w[0] < w[1]);
    if is_strictly_ascending {
        return true;
    }
    let is_strictly_descending = steps.windows(2).all(|w| w[0] > w[1]);
    if is_strictly_descending {
        return true;
    }

    // For count == 3, any other non-monotonic permutation is valid
    if count == 3 {
        return false;
    }

    // 2. Cyclic sweeps (circular left or right chase):
    // e.g. [1, 2, 3, 0] or [3, 0, 1, 2] where almost every step is +1 mod N or -1 mod N
    let forward_steps = steps
        .windows(2)
        .filter(|w| (w[1] + count - w[0]) % count == 1)
        .count();
    if forward_steps >= count - 1 {
        return true;
    }

    let backward_steps = steps
        .windows(2)
        .filter(|w| (w[0] + count - w[1]) % count == 1)
        .count();
    if backward_steps >= count - 1 {
        return true;
    }

    // 3. Wing / Center V-shape checks (for count >= 4):
    // Left half monotonic in one direction, right half monotonic in the opposite direction.
    let mid_left = count / 2;
    let mid_right = count.div_ceil(2);

    let left_half = &steps[..mid_left];
    let right_half = &steps[mid_right..];

    if left_half.len() >= 2 && right_half.len() >= 2 {
        let left_asc = left_half.windows(2).all(|w| w[0] < w[1]);
        let left_desc = left_half.windows(2).all(|w| w[0] > w[1]);
        let right_asc = right_half.windows(2).all(|w| w[0] < w[1]);
        let right_desc = right_half.windows(2).all(|w| w[0] > w[1]);

        // V-shape (wing: falls towards center, rises towards outside)
        if left_desc && right_asc {
            return true;
        }
        // Inverted V-shape (center: rises towards center, falls towards outside)
        if left_asc && right_desc {
            return true;
        }
    } else if count == 4 {
        // [3, 0, 1, 2] or [2, 0, 1, 3] or [3, 1, 0, 2] (V-shape)
        if steps[0] > steps[1]
            && steps[2] < steps[3]
            && steps[1].max(steps[2]) <= steps[0].min(steps[3])
        {
            return true;
        }
        // Inverted V-shape
        if steps[0] < steps[1]
            && steps[2] > steps[3]
            && steps[1].min(steps[2]) >= steps[0].max(steps[3])
        {
            return true;
        }
    }

    // 4. Interleaved wing / center:
    let is_outside_in = steps.iter().enumerate().all(|(i, &val)| {
        let expected = if i % 2 == 0 {
            i / 2
        } else {
            count - 1 - (i / 2)
        };
        val == expected
    });
    if is_outside_in {
        return true;
    }

    let is_outside_in_rev = steps.iter().enumerate().all(|(i, &val)| {
        let expected = if i % 2 == 0 {
            count - 1 - (i / 2)
        } else {
            i / 2
        };
        val == expected
    });
    if is_outside_in_rev {
        return true;
    }

    // 5. Symmetric wing mirror check (for count >= 6):
    if count >= 6 {
        let mut symmetric_pairs = 0;
        for i in 0..(count / 2) {
            let diff = (steps[i] as isize - steps[count - 1 - i] as isize).abs();
            if diff <= 1 {
                symmetric_pairs += 1;
            }
        }
        if symmetric_pairs == count / 2 {
            return true;
        }
    }

    // 6. Long consecutive run check (for count >= 6):
    if count >= 6 {
        let max_consecutive_run = max_sequential_run(steps);
        if max_consecutive_run >= (count / 2).max(3) {
            return true;
        }
    }

    false
}

/// Generates a deterministic shuffled permutation of `0..count` using SplitMix64
/// and Fisher-Yates shuffle with forced-random filtering.
/// Guaranteed:
/// - Every integer in `0..count` appears exactly once (zero duplicates, zero gaps).
/// - Evenly spreads the cycle phases across fixtures/pixels, matching grandMA2 shuffle.
/// - Forced random: rejects accidental linear sweeps (left / right) and symmetrical
///   V-shapes (wing / center), ensuring a truly scattered, chaotic feel.
pub fn generate_shuffled_steps(count: usize, seed: u64) -> Vec<usize> {
    if count <= 1 {
        return (0..count).collect();
    }

    let mut state = seed;
    let mut candidate: Vec<usize> = (0..count).collect();

    for _ in 0..64 {
        candidate = (0..count).collect();
        for i in (1..count).rev() {
            state = splitmix64(state);
            let j = (state as usize) % (i + 1);
            candidate.swap(i, j);
        }
        if !is_too_structured(&candidate) {
            return candidate;
        }
        // Advance seed state to try another shuffle
        state = splitmix64(state ^ 0x9e3779b97f4a7c15);
    }

    candidate
}

/// Calculates the normalized cyclic fraction ($0.0..1.0$) for an effect waveform.
///
/// Note on `"rng"` alignment: In `LiveEngine`, when alignment is `"rng"`, the
/// caller (`compute_fixture_dmx`) pre-maps `local_index` to a unique slot in the
/// group's cached grandMA2-style Fisher-Yates permutation via `engine.get_rng_step`
/// (which uses `generate_shuffled_steps` with forced-random filtering).
/// Thus, `local_index` arrives already assigned to its balanced phase slot in
/// `0..group_count`, and `(i / n) * 2π` yields that slot's phase offset.
pub fn calculate_effective_fract(
    effect_state: &GroupEffectState,
    elapsed: f32,
    local_index: usize,
    group_count: usize,
    fixture_id: u64,
    bpm: u16,
) -> Option<f32> {
    let _ = fixture_id;
    match effect_state.effect.as_deref() {
        Some(e) if e != "none" && e != "off" => e,
        _ => return None,
    };

    if bpm == 0 {
        return None;
    }

    let speed = (bpm as f32 / 60.0) * 2.0 * std::f32::consts::PI;
    let n = group_count.max(1) as f32;
    let i = local_index as f32;

    let phase_offset = match effect_state.alignment.as_deref().unwrap_or("single") {
        "single" | "none" => 0.0,
        "left" => (i / n) * 2.0 * std::f32::consts::PI,
        // "rng": `local_index` is pre-mapped by `LiveEngine::get_rng_step` into the
        // group's cached grandMA2-style shuffled permutation (via `generate_shuffled_steps`),
        // so `i` is already the shuffled slot index in `0..n`.
        "rng" => (i / n) * 2.0 * std::f32::consts::PI,
        "right" => (1.0 - (i / n)) * 2.0 * std::f32::consts::PI,
        "wing" => {
            let center_dist = (((i + 0.5) / n) - 0.5).abs() * 2.0;
            center_dist * 2.0 * std::f32::consts::PI
        }
        "center" => {
            let center_dist = (((i + 0.5) / n) - 0.5).abs() * 2.0;
            (1.0 - center_dist.clamp(0.0, 1.0)) * 2.0 * std::f32::consts::PI
        }
        "two" => (local_index % 2) as f32 * std::f32::consts::PI,
        "three" => (local_index % 3) as f32 * (2.0 * std::f32::consts::PI / 3.0),
        "four" => (local_index % 4) as f32 * (std::f32::consts::PI / 2.0),
        _ => 0.0,
    };

    let norm_t = (elapsed * speed + phase_offset) / (2.0 * std::f32::consts::PI);
    let cycle = norm_t.floor() as i32;
    let fract = norm_t.fract();

    let effective_fract = if effect_state.bounce && (cycle % 2 != 0) {
        1.0 - fract
    } else {
        fract
    };

    Some(effective_fract)
}

pub fn calculate_effect_factor(
    effect_state: &GroupEffectState,
    elapsed: f32,
    local_index: usize,
    group_count: usize,
    fixture_id: u64,
    bpm: u16,
) -> f32 {
    let effect_type = match effect_state.effect.as_deref() {
        Some(e) if e != "none" && e != "off" => e,
        _ => return 0.0,
    };

    if effect_type == "full" {
        return 1.0;
    }

    let Some(effective_fract) = calculate_effective_fract(
        effect_state,
        elapsed,
        local_index,
        group_count,
        fixture_id,
        bpm,
    ) else {
        return 0.0;
    };

    let w = (effect_state.width as f32 / 100.0).clamp(0.0, 1.0);
    if w <= 0.0 || effective_fract >= w {
        return 0.0;
    }

    let active_fract = effective_fract / w;
    let t = active_fract * 2.0 * std::f32::consts::PI;

    let val = match effect_type {
        "pulse" => {
            let sin_val = t.sin().max(0.0);
            sin_val.powf(2.5)
        }
        "chase" => {
            if t.sin() > 0.5 {
                1.0
            } else {
                0.0
            }
        }
        "ramp_up" => active_fract,
        "ramp_down" => 1.0 - active_fract,
        "sine" => (t.sin() + 1.0) * 0.5,
        _ => 1.0,
    };

    val.clamp(0.0, 1.0)
}

pub fn calculate_effect_intensity(
    effect_state: &GroupEffectState,
    elapsed: f32,
    local_index: usize,
    group_count: usize,
    fixture_id: u64,
    bpm: u16,
) -> u8 {
    (calculate_effect_factor(
        effect_state,
        elapsed,
        local_index,
        group_count,
        fixture_id,
        bpm,
    ) * 255.0)
        .round()
        .clamp(0.0, 255.0) as u8
}

pub fn compute_random_strobe_active(fixture_id: u64, elapsed: f32, rate: u8) -> bool {
    let rate_norm = (rate as f32 / 100.0).clamp(0.0, 1.0);
    // Interval between random strobe evaluation slots (50ms at rate=100 up to 400ms at rate=0)
    let step_time = 0.40 - rate_norm * 0.35;
    let slot = (elapsed / step_time).floor() as u64;
    let slot_fract = (elapsed / step_time).fract();

    let h = splitmix64(fixture_id ^ slot.wrapping_mul(0x9e3779b97f4a7c15));
    let rand_val = (h as f64 / u64::MAX as f64) as f32;

    let threshold = 0.35 + rate_norm * 0.25;
    rand_val < threshold && slot_fract < 0.45
}
