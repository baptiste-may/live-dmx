use crate::output::types::GroupEffectState;

pub fn calculate_effective_fract(
    effect_state: &GroupEffectState,
    elapsed: f32,
    local_index: usize,
    group_count: usize,
    fixture_id: u32,
    bpm: u16,
) -> Option<f32> {
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
        "rng" => {
            let hash = fixture_id.wrapping_mul(2654435761);
            (hash as f32 / u32::MAX as f32) * 2.0 * std::f32::consts::PI
        }
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
    fixture_id: u32,
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

    let t = effective_fract * 2.0 * std::f32::consts::PI;

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
        "ramp_up" => effective_fract,
        "ramp_down" => 1.0 - effective_fract,
        "sine" => (t.sin() + 1.0) * 0.5,
        "triangle" => {
            if effective_fract < 0.5 {
                effective_fract * 2.0
            } else {
                (1.0 - effective_fract) * 2.0
            }
        }
        "dip" if t.sin() > 0.5 => 0.0,
        _ => 1.0,
    };

    val.clamp(0.0, 1.0)
}

pub fn calculate_effect_intensity(
    effect_state: &GroupEffectState,
    elapsed: f32,
    local_index: usize,
    group_count: usize,
    fixture_id: u32,
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

pub fn compute_random_strobe_active(fixture_id: u32, elapsed: f32, rate: u8) -> bool {
    let rate_norm = (rate as f32 / 100.0).clamp(0.0, 1.0);
    // Interval between random strobe evaluation slots (50ms at rate=100 up to 400ms at rate=0)
    let step_time = 0.40 - rate_norm * 0.35;
    let slot = (elapsed / step_time).floor() as u64;
    let slot_fract = (elapsed / step_time).fract();

    // Deterministic pseudo-random hash using fixture_id and slot index
    let h = (fixture_id as u64)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(slot.wrapping_mul(1442695040888963407));
    let rand_val = ((h >> 32) as u32 as f32) / (u32::MAX as f32);

    let threshold = 0.35 + rate_norm * 0.25;
    rand_val < threshold && slot_fract < 0.45
}
