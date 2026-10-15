use std::collections::HashMap;
use std::time::Instant;

use crate::output::channels::get_fixture_channel_capabilities;
use crate::output::color_routing::{
    closest_color_position, effective_color_preference, map_temperature_to_dmx,
    rgb_to_color_temperature,
};
use crate::output::colors::{hsv_to_rgb, resolve_color_rgb};
use crate::output::effects::{
    calculate_effect_factor, calculate_effect_intensity, calculate_effective_fract,
    compute_random_strobe_active, generate_shuffled_steps, splitmix64,
};
use crate::output::pipeline::{encode_fixture_dmx, LogicalFixtureState};
use crate::output::profile::{
    compile_from_embedded_mode, compile_from_patched_fixture, CompiledFixtureProfile,
    LogicalAttribute,
};
use crate::output::types::{FixtureColorTransition, FlashFadeTransition, GroupEffectState};
use crate::show::{
    ColorPreference, EmbeddedGdtfMode, GroupSettings, PatchedFixture, PixelPreference, ShowData,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixturePixelStepInfo {
    pub step_offset: usize,
    pub step_count: usize,
    pub total_steps: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FixtureCacheKey {
    id: u32,
    groups: Vec<u32>,
    universe: u32,
    address: u32,
    mode: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct CachedFixture {
    pub id: u32,
    pub universe: u16,
    pub start_addr: usize,
    pub profile: CompiledFixtureProfile,
    pub groups: Vec<u32>,
    pub primary_group_id: u32,
    pub group_positions: HashMap<u32, (usize, usize)>,
    pub group_pixel_steps: HashMap<u32, FixturePixelStepInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FlashType {
    Simple,
    White,
    Master,
    Strobe,
    RandomStrobe,
    RandomStrobeAlt,
}

impl FlashType {
    fn from_str(s: &str) -> Option<FlashType> {
        match s {
            "simple" => Some(FlashType::Simple),
            "white" => Some(FlashType::White),
            "master" => Some(FlashType::Master),
            "strobe" => Some(FlashType::Strobe),
            "randomStrobe" => Some(FlashType::RandomStrobe),
            "random_strobe" => Some(FlashType::RandomStrobeAlt),
            _ => None,
        }
    }

    fn fade_duration(self, simple_fade: f32, white_fade: f32) -> f32 {
        match self {
            FlashType::Simple => simple_fade,
            FlashType::White | FlashType::Master => white_fade,
            _ => 0.0,
        }
    }
}

pub struct LiveEngine {
    pub flash_states: HashMap<u32, HashMap<String, bool>>,
    pub button_states: HashMap<String, bool>,
    pub group_effects: HashMap<u32, GroupEffectState>,
    pub group_color_effects: HashMap<u32, GroupEffectState>,
    pub group_rng_permutations: HashMap<u32, Vec<usize>>,
    pub group_color_rng_permutations: HashMap<u32, Vec<usize>>,
    pub rng_shuffle_seed: u64,
    pub group_masters: HashMap<u32, u8>,
    pub group_colors: HashMap<u32, Option<String>>,
    pub group_secondary_colors: HashMap<u32, Option<String>>,
    pub strobe_rate: u8,
    pub random_strobe_rate: u8,
    pub simple_flash_fade: f32,
    pub white_flash_fade: f32,
    flash_fade_transitions: HashMap<(u32, FlashType), FlashFadeTransition>,
    pub effect_bpm: u16,
    pub color_effect_bpm: u16,
    pub color_fade_speed: f32,
    pub color_fade_order: Option<String>,
    pub fixture_color_transitions: HashMap<u32, FixtureColorTransition>,
    pub fixture_last_colors: HashMap<u32, (u8, u8, u8)>,
    pub manual_channels: HashMap<(u16, u16), u8>,
    pub universe_buffers: HashMap<u16, Vec<u8>>,
    start_time: Instant,
    pub cached_fixtures: Vec<CachedFixture>,
    cached_keys: Vec<FixtureCacheKey>,
    cached_group_settings: HashMap<u32, GroupSettings>,
}

impl Default for LiveEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveEngine {
    pub fn new() -> Self {
        let mut universe_buffers = HashMap::new();
        for u in 1..=4 {
            universe_buffers.insert(u, vec![0u8; 512]);
        }

        let mut group_colors = HashMap::new();
        group_colors.insert(0, Some("white".to_string()));

        Self {
            flash_states: HashMap::new(),
            button_states: HashMap::new(),
            group_effects: HashMap::new(),
            group_color_effects: HashMap::new(),
            group_rng_permutations: HashMap::new(),
            group_color_rng_permutations: HashMap::new(),
            rng_shuffle_seed: 0x123456789abcdef0,
            group_masters: HashMap::new(),
            group_colors,
            group_secondary_colors: HashMap::new(),
            strobe_rate: 50,
            random_strobe_rate: 50,
            simple_flash_fade: 0.0,
            white_flash_fade: 0.0,
            flash_fade_transitions: HashMap::new(),
            effect_bpm: 60,
            color_effect_bpm: 60,
            color_fade_speed: 1.0,
            color_fade_order: None,
            fixture_color_transitions: HashMap::new(),
            fixture_last_colors: HashMap::new(),
            manual_channels: HashMap::new(),
            universe_buffers,
            start_time: Instant::now(),
            cached_fixtures: Vec::new(),
            cached_keys: Vec::new(),
            cached_group_settings: HashMap::new(),
        }
    }

    pub fn set_flash(&mut self, group_id: u32, flash_type: String, active: bool) {
        let group_flash = self.flash_states.entry(group_id).or_default();
        let was_active = *group_flash.get(&flash_type).unwrap_or(&false);
        group_flash.insert(flash_type.clone(), active);

        let Some(ft) = FlashType::from_str(&flash_type) else {
            return;
        };
        if active {
            self.flash_fade_transitions.remove(&(group_id, ft));
        } else if was_active {
            let fade_duration = ft.fade_duration(self.simple_flash_fade, self.white_flash_fade);
            if fade_duration > 0.0 {
                self.flash_fade_transitions.insert(
                    (group_id, ft),
                    FlashFadeTransition {
                        start_time: Instant::now(),
                        duration_secs: fade_duration,
                    },
                );
            }
        }
    }

    pub fn set_simple_flash_fade(&mut self, fade: f32) {
        self.simple_flash_fade = fade.max(0.0);
    }

    pub fn set_white_flash_fade(&mut self, fade: f32) {
        self.white_flash_fade = fade.max(0.0);
    }

    pub fn get_flash_factor(&self, group_id: u32, flash_type: &str) -> f32 {
        if self.is_flash_active(group_id, flash_type) {
            return 1.0;
        }
        let Some(ft) = FlashType::from_str(flash_type) else {
            return 0.0;
        };
        if let Some(trans) = self.flash_fade_transitions.get(&(group_id, ft)) {
            let elapsed = trans.start_time.elapsed().as_secs_f32();
            if elapsed < trans.duration_secs && trans.duration_secs > 0.0 {
                return (1.0 - elapsed / trans.duration_secs).clamp(0.0, 1.0);
            }
        }
        if group_id != 0 {
            if let Some(trans) = self.flash_fade_transitions.get(&(0, ft)) {
                let elapsed = trans.start_time.elapsed().as_secs_f32();
                if elapsed < trans.duration_secs && trans.duration_secs > 0.0 {
                    return (1.0 - elapsed / trans.duration_secs).clamp(0.0, 1.0);
                }
            }
        }
        0.0
    }

    pub fn set_button(&mut self, button_id: String, active: bool) {
        self.button_states.insert(button_id, active);
    }

    pub fn set_group_effect(
        &mut self,
        group_id: u32,
        effect: Option<String>,
        alignment: Option<String>,
        bounce: bool,
    ) {
        if alignment.as_deref() == Some("rng") {
            self.rng_shuffle_seed = splitmix64(self.rng_shuffle_seed);
            self.group_rng_permutations.remove(&group_id);
        }
        let entry = self.group_effects.entry(group_id).or_default();
        entry.effect = effect;
        entry.alignment = alignment;
        entry.bounce = bounce;
    }

    pub fn set_group_effect_width(&mut self, group_id: u32, width: u8) {
        let entry = self.group_effects.entry(group_id).or_default();
        entry.width = width.min(100);
    }

    pub fn set_group_master(&mut self, group_id: u32, level: u8) {
        self.group_masters.insert(group_id, level);
    }

    pub fn get_group_master(&self, group_id: u32) -> u8 {
        *self.group_masters.get(&group_id).unwrap_or(&100)
    }

    pub fn set_effect_bpm(&mut self, bpm: u16) {
        self.effect_bpm = bpm;
    }

    pub fn set_color_effect_bpm(&mut self, bpm: u16) {
        self.color_effect_bpm = bpm;
    }

    pub fn set_color_fade_speed(&mut self, speed: f32) {
        self.color_fade_speed = speed.max(0.0);
    }

    pub fn set_color_fade_order(&mut self, order: Option<String>) {
        self.color_fade_order = order;
    }

    pub fn get_fixture_current_rgb_f32(
        &self,
        fixture_id: u32,
        fallback_str: Option<&str>,
    ) -> (f32, f32, f32) {
        if let Some(transition) = self.fixture_color_transitions.get(&fixture_id) {
            let elapsed = transition.start_time.elapsed().as_secs_f32();
            if elapsed < transition.delay_secs {
                transition.start_rgb
            } else if elapsed >= transition.delay_secs + transition.duration_secs
                || transition.duration_secs <= 0.0
            {
                transition.target_rgb
            } else {
                let progress = (elapsed - transition.delay_secs) / transition.duration_secs;
                (
                    transition.start_rgb.0 * (1.0 - progress) + transition.target_rgb.0 * progress,
                    transition.start_rgb.1 * (1.0 - progress) + transition.target_rgb.1 * progress,
                    transition.start_rgb.2 * (1.0 - progress) + transition.target_rgb.2 * progress,
                )
            }
        } else if let Some(&(r, g, b)) = self.fixture_last_colors.get(&fixture_id) {
            (r as f32, g as f32, b as f32)
        } else {
            let (r, g, b) = match fallback_str {
                Some(s) => resolve_color_rgb(s),
                None => (255, 255, 255),
            };
            (r as f32, g as f32, b as f32)
        }
    }

    pub fn set_group_color(
        &mut self,
        group_id: u32,
        color: Option<String>,
        show: Option<&ShowData>,
    ) {
        if let Some(c) = &color {
            self.group_colors.insert(group_id, Some(c.clone()));
        } else {
            self.group_colors.remove(&group_id);
        }

        let target_rgb = match &color {
            Some(c) => resolve_color_rgb(c),
            None => (255, 255, 255),
        };
        let target_rgb_f = (
            target_rgb.0 as f32,
            target_rgb.1 as f32,
            target_rgb.2 as f32,
        );

        let Some(show_data) = show else {
            return;
        };

        let fixtures: Vec<&PatchedFixture> = show_data
            .patch
            .fixtures
            .iter()
            .filter(|f| {
                if group_id == 0 {
                    true
                } else {
                    f.groups.contains(&group_id)
                }
            })
            .collect();

        if fixtures.is_empty() {
            return;
        }

        if self.color_fade_speed <= 0.0 || color.is_none() {
            for f in fixtures {
                self.fixture_color_transitions.remove(&f.id);
                self.fixture_last_colors.insert(f.id, target_rgb);
            }
            return;
        }

        let n = fixtures.len();
        let speed = self.color_fade_speed;
        let now = Instant::now();

        for (i, f) in fixtures.into_iter().enumerate() {
            let fallback_str = f
                .groups
                .iter()
                .find_map(|&g| self.get_group_color(g))
                .or_else(|| self.get_group_color(0))
                .or(Some("white"));
            let current_rgb_f = self.get_fixture_current_rgb_f32(f.id, fallback_str);

            let (delay_secs, duration_secs) = match self.color_fade_order.as_deref() {
                None => (0.0, speed),
                Some(order) => {
                    let p_i = if n <= 1 {
                        0.0
                    } else {
                        match order {
                            "left" => (i as f32) / ((n - 1) as f32),
                            "right" => ((n - 1 - i) as f32) / ((n - 1) as f32),
                            "wing" => (((i as f32 + 0.5) / (n as f32)) - 0.5).abs() * 2.0,
                            "center" => (1.0 - (((i as f32 + 0.5) / (n as f32)) - 0.5).abs() * 2.0)
                                .clamp(0.0, 1.0),
                            _ => 0.0,
                        }
                    }
                    .clamp(0.0, 1.0);

                    let indiv_duration = (speed * 0.5).max(0.05);
                    let max_delay = (speed - indiv_duration).max(0.0);
                    (p_i * max_delay, indiv_duration)
                }
            };

            self.fixture_color_transitions.insert(
                f.id,
                FixtureColorTransition {
                    start_rgb: current_rgb_f,
                    target_rgb: target_rgb_f,
                    start_time: now,
                    delay_secs,
                    duration_secs,
                },
            );
        }
    }

    pub fn set_all_groups_color(&mut self, color: Option<String>, show: Option<&ShowData>) {
        if let Some(c) = color {
            self.group_colors.insert(0, Some(c.clone()));
            for val in self.group_colors.values_mut() {
                *val = Some(c.clone());
            }
            self.set_group_color(0, Some(c), show);
        } else {
            self.group_colors.clear();
            self.group_colors.insert(0, Some("white".to_string()));
            self.set_group_color(0, Some("white".to_string()), show);
        }
    }

    pub fn get_group_color(&self, group_id: u32) -> Option<&str> {
        if let Some(Some(c)) = self.group_colors.get(&group_id) {
            return Some(c.as_str());
        }
        None
    }

    pub fn set_group_secondary_color(&mut self, group_id: u32, color: Option<String>) {
        if let Some(c) = color {
            self.group_secondary_colors.insert(group_id, Some(c));
        } else {
            self.group_secondary_colors.remove(&group_id);
        }
    }

    pub fn get_group_secondary_color(&self, group_id: u32) -> Option<&str> {
        if let Some(Some(c)) = self.group_secondary_colors.get(&group_id) {
            return Some(c.as_str());
        }
        if group_id != 0 {
            if let Some(Some(c)) = self.group_secondary_colors.get(&0) {
                return Some(c.as_str());
            }
        }
        None
    }

    pub fn set_group_color_effect(
        &mut self,
        group_id: u32,
        effect: Option<String>,
        alignment: Option<String>,
        bounce: bool,
    ) {
        if alignment.as_deref() == Some("rng") {
            self.rng_shuffle_seed = splitmix64(self.rng_shuffle_seed);
            self.group_color_rng_permutations.remove(&group_id);
        }
        let entry = self.group_color_effects.entry(group_id).or_default();
        entry.effect = effect;
        entry.alignment = alignment;
        entry.bounce = bounce;
    }

    pub fn set_group_color_effect_width(&mut self, group_id: u32, width: u8) {
        let entry = self.group_color_effects.entry(group_id).or_default();
        entry.width = width.min(100);
    }

    pub fn get_group_color_effect(&self, group_id: u32) -> GroupEffectState {
        self.group_color_effects
            .get(&group_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn reset_group_color_effect(&mut self, group_id: u32) {
        self.group_color_effects
            .insert(group_id, GroupEffectState::default());
        self.group_color_rng_permutations.remove(&group_id);
    }

    pub fn reset_group_effect(&mut self, group_id: u32) {
        self.group_effects
            .insert(group_id, GroupEffectState::default());
        self.group_rng_permutations.remove(&group_id);
    }

    pub fn get_rng_step(&mut self, group_id: u32, step: usize, total_steps: usize) -> usize {
        if total_steps <= 1 {
            return 0;
        }
        let seed = self.rng_shuffle_seed ^ (group_id as u64);
        let perm = self
            .group_rng_permutations
            .entry(group_id)
            .or_insert_with(|| generate_shuffled_steps(total_steps, seed));
        if perm.len() != total_steps {
            *perm = generate_shuffled_steps(total_steps, seed);
        }
        perm.get(step).copied().unwrap_or(step)
    }

    pub fn get_color_rng_step(&mut self, group_id: u32, step: usize, total_steps: usize) -> usize {
        if total_steps <= 1 {
            return 0;
        }
        let seed = self.rng_shuffle_seed.wrapping_add(1) ^ (group_id as u64);
        let perm = self
            .group_color_rng_permutations
            .entry(group_id)
            .or_insert_with(|| generate_shuffled_steps(total_steps, seed));
        if perm.len() != total_steps {
            *perm = generate_shuffled_steps(total_steps, seed);
        }
        perm.get(step).copied().unwrap_or(step)
    }

    pub fn reset_all(&mut self) {
        self.flash_states.clear();
        self.button_states.clear();
        self.group_effects.clear();
        self.group_color_effects.clear();
        self.group_rng_permutations.clear();
        self.group_color_rng_permutations.clear();
        self.group_masters.clear();
        self.group_colors.clear();
        self.group_colors.insert(0, Some("white".to_string()));
        self.group_secondary_colors.clear();
        self.strobe_rate = 50;
        self.random_strobe_rate = 50;
        self.simple_flash_fade = 0.0;
        self.white_flash_fade = 0.0;
        self.flash_fade_transitions.clear();
        self.effect_bpm = 60;
        self.color_effect_bpm = 60;
        self.color_fade_speed = 1.0;
        self.color_fade_order = None;
        self.fixture_color_transitions.clear();
        self.fixture_last_colors.clear();
        self.manual_channels.clear();
        self.cached_fixtures.clear();
        self.cached_keys.clear();
        self.cached_group_settings.clear();
        for buf in self.universe_buffers.values_mut() {
            buf.fill(0);
        }
    }

    pub fn is_flash_active(&self, group_id: u32, flash_type: &str) -> bool {
        if let Some(types) = self.flash_states.get(&group_id) {
            if *types.get(flash_type).unwrap_or(&false) {
                return true;
            }
        }
        if group_id != 0 {
            if let Some(types) = self.flash_states.get(&0) {
                if *types.get(flash_type).unwrap_or(&false) {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_group_effect(&self, group_id: u32) -> GroupEffectState {
        self.group_effects
            .get(&group_id)
            .cloned()
            .unwrap_or_default()
    }

    fn build_cached_fixture(
        f: &PatchedFixture,
        start_addr: usize,
        primary_group_id: u32,
        group_positions: HashMap<u32, (usize, usize)>,
        show_data: &ShowData,
    ) -> CachedFixture {
        let mode_name = f.mode.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let fix_type_name = f
            .fixture_type
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let gdtf_spec = f
            .fixture_type
            .get("gdtfSpec")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let fix_key = f
            .fixture_type
            .get("fixtureKey")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mfg_key = f
            .fixture_type
            .get("manufacturerKey")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let direct_key = format!(
            "{}@{}",
            mfg_key.replace('_', " ").to_lowercase(),
            fix_type_name.to_lowercase()
        );
        let direct_def = show_data.fixture_definitions.get(&direct_key).or_else(|| {
            show_data
                .fixture_definitions
                .get(&format!("{}@{}", mfg_key, fix_key))
        });

        let matched_def_mode: Option<&EmbeddedGdtfMode> = direct_def
            .and_then(|def| {
                def.modes
                    .iter()
                    .find(|m| m.name.eq_ignore_ascii_case(mode_name))
            })
            .or_else(|| {
                let mut matches: Vec<_> = show_data
                    .fixture_definitions
                    .iter()
                    .filter(|(_, def)| {
                        let def_name_match = !fix_type_name.is_empty()
                            && def.name.eq_ignore_ascii_case(fix_type_name);
                        let gdtf_spec_match =
                            !gdtf_spec.is_empty() && gdtf_spec.eq_ignore_ascii_case(&def.name);
                        let key_match = !fix_key.is_empty()
                            && def.name.to_lowercase().replace(' ', "_") == fix_key.to_lowercase();
                        def_name_match || gdtf_spec_match || key_match
                    })
                    .collect();
                matches.sort_by_key(|(k, _)| (*k).clone());
                matches.into_iter().find_map(|(_, def)| {
                    def.modes
                        .iter()
                        .find(|m| m.name.eq_ignore_ascii_case(mode_name))
                })
            });

        let profile = if let Some(m) = matched_def_mode {
            let mut merged_mode = m.clone();
            if let Some(defaults_arr) = f.mode.get("channelDefaults").and_then(|v| v.as_array()) {
                let d_vec: Vec<u8> = defaults_arr
                    .iter()
                    .map(|v| v.as_u64().unwrap_or(0) as u8)
                    .collect();
                if !d_vec.is_empty() {
                    merged_mode.channel_defaults = Some(d_vec);
                }
            }
            if let Some(shutter_obj) = f.mode.get("shutterOpenValues").and_then(|v| v.as_object()) {
                let mut s_map = merged_mode.shutter_open_values.unwrap_or_default();
                for (k, v) in shutter_obj {
                    if let Ok(idx) = k.parse::<usize>() {
                        s_map.insert(idx, v.as_u64().unwrap_or(255) as u8);
                    }
                }
                merged_mode.shutter_open_values = Some(s_map);
            }
            let caps = (0..merged_mode.channels.len())
                .map(|ch| get_fixture_channel_capabilities(f, ch))
                .collect::<Vec<_>>();
            compile_from_embedded_mode(&merged_mode, Some(&caps))
        } else {
            compile_from_patched_fixture(f)
        };

        CachedFixture {
            id: f.id,
            universe: f.universe as u16,
            start_addr,
            profile,
            groups: f.groups.clone(),
            primary_group_id,
            group_positions,
            group_pixel_steps: HashMap::new(),
        }
    }

    fn ensure_fixtures_cache(&mut self, show_data: &ShowData) {
        let fixtures = &show_data.patch.fixtures;
        let is_cache_valid = self.cached_keys.len() == fixtures.len()
            && self.cached_group_settings == show_data.groups.settings
            && self.cached_keys.iter().zip(fixtures.iter()).all(|(k, f)| {
                k.id == f.id
                    && k.groups == f.groups
                    && k.universe == f.universe
                    && k.address == f.address
                    && k.mode == f.mode
            });

        if is_cache_valid {
            return;
        }

        let mut group_fixture_map: HashMap<u32, Vec<u32>> = HashMap::new();
        for f in fixtures {
            group_fixture_map.entry(0).or_default().push(f.id);
            for &g in &f.groups {
                if g != 0 {
                    group_fixture_map.entry(g).or_default().push(f.id);
                }
            }
        }

        let mut new_cached = Vec::with_capacity(fixtures.len());
        let mut new_keys = Vec::with_capacity(fixtures.len());

        for f in fixtures {
            let primary_group_id = f.groups.first().copied().unwrap_or(0);
            let mut group_positions = HashMap::new();

            let empty_vec = Vec::new();
            let g0_fixtures = group_fixture_map.get(&0).unwrap_or(&empty_vec);
            let pos0 = g0_fixtures.iter().position(|&id| id == f.id).unwrap_or(0);
            group_positions.insert(0, (pos0, g0_fixtures.len()));

            for &g in &f.groups {
                let g_fixtures = group_fixture_map.get(&g).unwrap_or(&empty_vec);
                let pos = g_fixtures.iter().position(|&id| id == f.id).unwrap_or(0);
                group_positions.insert(g, (pos, g_fixtures.len()));
            }

            let start_addr = (f.address.saturating_sub(1)) as usize;
            new_cached.push(Self::build_cached_fixture(
                f,
                start_addr,
                primary_group_id,
                group_positions,
                show_data,
            ));

            new_keys.push(FixtureCacheKey {
                id: f.id,
                groups: f.groups.clone(),
                universe: f.universe,
                address: f.address,
                mode: f.mode.clone(),
            });
        }

        // Compute group_pixel_steps for each group
        for (&group_id, f_ids) in &group_fixture_map {
            let (pref, group_count_setting) = match show_data.groups.settings.get(&group_id) {
                Some(s) => (s.pixel_preference, s.pixel_group_count),
                None => (PixelPreference::Single, 2),
            };

            let mut step_counts = Vec::with_capacity(f_ids.len());
            for &fid in f_ids {
                let p_count = new_cached
                    .iter()
                    .find(|cf| cf.id == fid)
                    .map(|cf| cf.profile.pixel_count)
                    .unwrap_or(1)
                    .max(1);
                let steps = match pref {
                    PixelPreference::Single => 1,
                    PixelPreference::Individual => p_count,
                    PixelPreference::Grouped => (group_count_setting as usize).min(p_count).max(1),
                };
                step_counts.push(steps);
            }

            let total_steps: usize = step_counts.iter().sum();
            let mut offset = 0;
            for (idx, &fid) in f_ids.iter().enumerate() {
                let s_i = step_counts[idx];
                if let Some(cf) = new_cached.iter_mut().find(|cf| cf.id == fid) {
                    cf.group_pixel_steps.insert(
                        group_id,
                        FixturePixelStepInfo {
                            step_offset: offset,
                            step_count: s_i,
                            total_steps,
                        },
                    );
                }
                offset += s_i;
            }
        }

        self.cached_fixtures = new_cached;
        self.cached_keys = new_keys;
        self.cached_group_settings = show_data.groups.settings.clone();
    }

    pub fn compute_dmx(&mut self, show_data: &ShowData) {
        for u_cfg in &show_data.output.universes {
            self.universe_buffers
                .entry(u_cfg.universe as u16)
                .or_insert_with(|| vec![0u8; 512]);
        }
        for f in &show_data.patch.fixtures {
            self.universe_buffers
                .entry(f.universe as u16)
                .or_insert_with(|| vec![0u8; 512]);
        }

        for buf in self.universe_buffers.values_mut() {
            buf.fill(0);
        }

        self.ensure_fixtures_cache(show_data);

        let elapsed = self.start_time.elapsed().as_secs_f32();
        let cached_fixtures = std::mem::take(&mut self.cached_fixtures);

        for fixture in &cached_fixtures {
            let primary_group_id = fixture.primary_group_id;

            let group_effect_state = self.get_group_effect(primary_group_id);
            let global_effect_state = self.get_group_effect(0);

            let (active_group_id, effect_state) = if group_effect_state.effect.is_some() {
                (primary_group_id, group_effect_state)
            } else {
                (0, global_effect_state)
            };

            let group_color_effect_state = self.get_group_color_effect(primary_group_id);
            let global_color_effect_state = self.get_group_color_effect(0);

            let (active_color_group_id, color_effect_state) =
                if group_color_effect_state.effect.is_some() {
                    (primary_group_id, group_color_effect_state)
                } else {
                    (0, global_color_effect_state)
                };

            let (local_index, group_count) = *fixture
                .group_positions
                .get(&active_group_id)
                .unwrap_or(&(0, 1));

            let (color_local_index, color_group_count) = *fixture
                .group_positions
                .get(&active_color_group_id)
                .unwrap_or(&(0, 1));

            let intensity_pixel_steps = fixture
                .group_pixel_steps
                .get(&active_group_id)
                .copied()
                .unwrap_or(FixturePixelStepInfo {
                    step_offset: local_index,
                    step_count: 1,
                    total_steps: group_count,
                });

            let color_pixel_steps = fixture
                .group_pixel_steps
                .get(&active_color_group_id)
                .copied()
                .unwrap_or(FixturePixelStepInfo {
                    step_offset: color_local_index,
                    step_count: 1,
                    total_steps: color_group_count,
                });

            self.compute_fixture_dmx(
                fixture,
                active_group_id,
                &effect_state,
                active_color_group_id,
                &color_effect_state,
                elapsed,
                &show_data.groups.settings,
                self.resolve_color_preference(fixture, &show_data.groups.settings),
                intensity_pixel_steps,
                color_pixel_steps,
            );
        }

        self.cached_fixtures = cached_fixtures;

        for (&(universe, channel), &val) in &self.manual_channels {
            if universe >= 1 && (1..=512).contains(&channel) {
                let buf = self
                    .universe_buffers
                    .entry(universe)
                    .or_insert_with(|| vec![0u8; 512]);
                buf[(channel - 1) as usize] = val;
            }
        }

        self.fixture_color_transitions.retain(|_, trans| {
            let elapsed = trans.start_time.elapsed().as_secs_f32();
            elapsed < trans.delay_secs + trans.duration_secs
        });

        self.flash_fade_transitions
            .retain(|_, trans| trans.start_time.elapsed().as_secs_f32() < trans.duration_secs);
    }

    fn resolve_color_preference(
        &self,
        fixture: &CachedFixture,
        settings: &HashMap<u32, GroupSettings>,
    ) -> ColorPreference {
        fixture
            .groups
            .iter()
            .chain(std::iter::once(&0))
            .find_map(|g| settings.get(g).map(|s| s.color_preference))
            .unwrap_or_default()
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_fixture_dmx(
        &mut self,
        fixture: &CachedFixture,
        active_group_id: u32,
        effect_state: &GroupEffectState,
        active_color_group_id: u32,
        color_effect_state: &GroupEffectState,
        elapsed: f32,
        settings: &HashMap<u32, GroupSettings>,
        color_preference: ColorPreference,
        intensity_pixel_steps: FixturePixelStepInfo,
        color_pixel_steps: FixturePixelStepInfo,
    ) {
        if fixture.start_addr >= 512 {
            return;
        }

        let effective_preference =
            effective_color_preference(color_preference, fixture.profile.color_summary);

        let simple_factor = fixture
            .groups
            .iter()
            .chain(std::iter::once(&0))
            .map(|&g| self.get_flash_factor(g, "simple"))
            .fold(0.0, f32::max);

        let white_factor = fixture
            .groups
            .iter()
            .chain(std::iter::once(&0))
            .map(|&g| {
                self.get_flash_factor(g, "white")
                    .max(self.get_flash_factor(g, "master"))
            })
            .fold(0.0, f32::max);

        let flash_dimmer_factor = simple_factor.max(white_factor);

        let is_in_strobe_group = fixture
            .groups
            .iter()
            .any(|&g| self.is_flash_active(g, "strobe"))
            || self.is_flash_active(0, "strobe");

        let is_in_random_strobe_group = fixture.groups.iter().any(|&g| {
            self.is_flash_active(g, "randomStrobe") || self.is_flash_active(g, "random_strobe")
        }) || self.is_flash_active(0, "randomStrobe")
            || self.is_flash_active(0, "random_strobe");

        let fixture_u64_id = (fixture.id as u64) << 32;

        let is_in_strobe = is_in_strobe_group || is_in_random_strobe_group;

        let strobe_val = if is_in_strobe_group {
            let freq = 1.0 + (self.strobe_rate as f32 / 100.0) * 19.0;
            if (elapsed * freq * 2.0 * std::f32::consts::PI).sin() > 0.0 {
                255
            } else {
                0
            }
        } else if is_in_random_strobe_group {
            if compute_random_strobe_active(fixture_u64_id, elapsed, self.random_strobe_rate) {
                255
            } else {
                0
            }
        } else {
            0
        };

        // A fixture without groups has `primary_group_id == 0`: the limit is then
        // already applied through `global_max_power` and must not be squared.
        let group_max_power = if fixture.primary_group_id == 0 {
            100
        } else {
            settings
                .get(&fixture.primary_group_id)
                .map_or(100u8, |s| s.max_power)
        };
        let global_max_power = settings.get(&0).map_or(100u8, |s| s.max_power);

        let group_master = (self.get_group_master(fixture.primary_group_id) as f32 / 100.0)
            * (group_max_power as f32 / 100.0);
        let global_master =
            (self.get_group_master(0) as f32 / 100.0) * (global_max_power as f32 / 100.0);
        let master_multiplier = group_master * global_master;
        // Absolute ceiling from the group settings only (masters excluded):
        // flash and strobe boosts must not exceed the configured max power.
        let max_power_factor = (group_max_power as f32 / 100.0) * (global_max_power as f32 / 100.0);

        let intensity_total = intensity_pixel_steps.total_steps.max(1);
        let effective_intensity_index = if effect_state.alignment.as_deref() == Some("rng") {
            self.get_rng_step(
                active_group_id,
                intensity_pixel_steps.step_offset,
                intensity_total,
            )
        } else {
            intensity_pixel_steps.step_offset
        };

        let raw_intensity = calculate_effect_intensity(
            effect_state,
            elapsed,
            effective_intensity_index,
            intensity_total,
            fixture_u64_id,
            self.effect_bpm,
        );

        let intensity_level = ((raw_intensity as f32) * master_multiplier).clamp(0.0, 255.0) as u8;
        let intensity_norm = (intensity_level as f32) / 255.0;

        let selected_color_str = fixture
            .groups
            .iter()
            .find_map(|&g| self.get_group_color(g))
            .or_else(|| self.get_group_color(0))
            .or(Some("white"));

        let current_rgb_f = self.get_fixture_current_rgb_f32(fixture.id, selected_color_str);

        let cur_r = current_rgb_f.0.round().clamp(0.0, 255.0) as u8;
        let cur_g = current_rgb_f.1.round().clamp(0.0, 255.0) as u8;
        let cur_b = current_rgb_f.2.round().clamp(0.0, 255.0) as u8;
        self.fixture_last_colors
            .insert(fixture.id, (cur_r, cur_g, cur_b));

        let secondary_color_str = fixture
            .groups
            .iter()
            .find_map(|&g| self.get_group_secondary_color(g))
            .or_else(|| self.get_group_secondary_color(0));

        let secondary_rgb = match secondary_color_str {
            Some(s) => resolve_color_rgb(s),
            None => (0, 0, 0),
        };
        let secondary_rgb_f = (
            secondary_rgb.0 as f32,
            secondary_rgb.1 as f32,
            secondary_rgb.2 as f32,
        );

        let color_total = color_pixel_steps.total_steps.max(1);
        let effective_color_index = if color_effect_state.alignment.as_deref() == Some("rng") {
            self.get_color_rng_step(
                active_color_group_id,
                color_pixel_steps.step_offset,
                color_total,
            )
        } else {
            color_pixel_steps.step_offset
        };

        let (r_f, g_f, b_f) = if color_effect_state.effect.as_deref() == Some("rainbow") {
            if let Some(fract) = calculate_effective_fract(
                color_effect_state,
                elapsed,
                effective_color_index,
                color_total,
                fixture_u64_id,
                self.color_effect_bpm,
            ) {
                let w = (color_effect_state.width as f32 / 100.0).clamp(0.0, 1.0);
                if w <= 0.0 || fract >= w {
                    current_rgb_f
                } else {
                    hsv_to_rgb(fract / w, 1.0, 1.0)
                }
            } else {
                current_rgb_f
            }
        } else {
            let color_factor = calculate_effect_factor(
                color_effect_state,
                elapsed,
                effective_color_index,
                color_total,
                fixture_u64_id,
                self.color_effect_bpm,
            );

            (
                current_rgb_f.0 * (1.0 - color_factor) + secondary_rgb_f.0 * color_factor,
                current_rgb_f.1 * (1.0 - color_factor) + secondary_rgb_f.1 * color_factor,
                current_rgb_f.2 * (1.0 - color_factor) + secondary_rgb_f.2 * color_factor,
            )
        };

        let r_norm = (r_f / 255.0).clamp(0.0, 1.0);
        let g_norm = (g_f / 255.0).clamp(0.0, 1.0);
        let b_norm = (b_f / 255.0).clamp(0.0, 1.0);

        let white_accent = if r_norm >= 0.784 && g_norm >= 0.784 && b_norm >= 0.784 {
            1.0
        } else {
            0.0
        };

        let r_byte = (r_norm * 255.0).round().clamp(0.0, 255.0) as u8;
        let g_byte = (g_norm * 255.0).round().clamp(0.0, 255.0) as u8;
        let b_byte = (b_norm * 255.0).round().clamp(0.0, 255.0) as u8;

        let color_wheel_dmx = if effective_preference == ColorPreference::ColorWheel {
            fixture
                .profile
                .channels
                .iter()
                .find(|c| c.attribute == LogicalAttribute::ColorWheel)
                .and_then(|c| {
                    let offset = c.resolution.primary_offset();
                    fixture
                        .profile
                        .channel_capabilities
                        .get(offset)
                        .map(|caps| closest_color_position(r_byte, g_byte, b_byte, caps))
                })
        } else {
            None
        };

        let color_temp_dmx = if effective_preference == ColorPreference::Cct {
            fixture
                .profile
                .channels
                .iter()
                .any(|c| c.attribute == LogicalAttribute::ColorTemperature)
                .then(|| map_temperature_to_dmx(rgb_to_color_temperature(r_byte, g_byte, b_byte)))
        } else {
            None
        };

        let color_macro_dmx = if effective_preference == ColorPreference::ColorPresets {
            fixture
                .profile
                .channels
                .iter()
                .find(|c| c.attribute == LogicalAttribute::ColorMacro)
                .and_then(|c| {
                    let offset = c.resolution.primary_offset();
                    fixture
                        .profile
                        .channel_capabilities
                        .get(offset)
                        .map(|caps| closest_color_position(r_byte, g_byte, b_byte, caps))
                })
        } else {
            None
        };

        let state = LogicalFixtureState {
            intensity: intensity_norm,
            max_power_factor,
            color_rgb: (r_norm, g_norm, b_norm),
            white_accent,
            flash_dimmer_factor,
            white_flash_factor: white_factor,
            is_in_strobe,
            strobe_val,
            color_wheel_dmx,
            color_macro_dmx,
            color_temp_dmx,
            pan_norm: None,
            tilt_norm: None,
        };

        let pixel_states = if fixture.profile.pixel_count > 1
            && (intensity_pixel_steps.step_count > 1 || color_pixel_steps.step_count > 1)
        {
            let mut states = Vec::with_capacity(fixture.profile.pixel_count);
            for p in 0..fixture.profile.pixel_count {
                let p_intensity_step_raw = intensity_pixel_steps.step_offset
                    + (p * intensity_pixel_steps.step_count) / fixture.profile.pixel_count;
                let p_intensity_total = intensity_pixel_steps.total_steps.max(1);
                let p_intensity_step = if effect_state.alignment.as_deref() == Some("rng") {
                    self.get_rng_step(active_group_id, p_intensity_step_raw, p_intensity_total)
                } else {
                    p_intensity_step_raw
                };

                let p_color_step_raw = color_pixel_steps.step_offset
                    + (p * color_pixel_steps.step_count) / fixture.profile.pixel_count;
                let p_color_total = color_pixel_steps.total_steps.max(1);
                let p_color_step = if color_effect_state.alignment.as_deref() == Some("rng") {
                    self.get_color_rng_step(active_color_group_id, p_color_step_raw, p_color_total)
                } else {
                    p_color_step_raw
                };

                let p_id = ((fixture.id as u64) << 32) | (p as u64 + 1);

                let p_raw_intensity = calculate_effect_intensity(
                    effect_state,
                    elapsed,
                    p_intensity_step,
                    p_intensity_total,
                    p_id,
                    self.effect_bpm,
                );
                let p_intensity_level =
                    ((p_raw_intensity as f32) * master_multiplier).clamp(0.0, 255.0) as u8;
                let p_intensity_norm = (p_intensity_level as f32) / 255.0;

                let (p_rf, p_gf, p_bf) = if color_effect_state.effect.as_deref() == Some("rainbow")
                {
                    if let Some(fract) = calculate_effective_fract(
                        color_effect_state,
                        elapsed,
                        p_color_step,
                        p_color_total,
                        p_id,
                        self.color_effect_bpm,
                    ) {
                        let w = (color_effect_state.width as f32 / 100.0).clamp(0.0, 1.0);
                        if w <= 0.0 || fract >= w {
                            current_rgb_f
                        } else {
                            hsv_to_rgb(fract / w, 1.0, 1.0)
                        }
                    } else {
                        current_rgb_f
                    }
                } else {
                    let color_factor = calculate_effect_factor(
                        color_effect_state,
                        elapsed,
                        p_color_step,
                        p_color_total,
                        p_id,
                        self.color_effect_bpm,
                    );

                    (
                        current_rgb_f.0 * (1.0 - color_factor) + secondary_rgb_f.0 * color_factor,
                        current_rgb_f.1 * (1.0 - color_factor) + secondary_rgb_f.1 * color_factor,
                        current_rgb_f.2 * (1.0 - color_factor) + secondary_rgb_f.2 * color_factor,
                    )
                };

                let p_r_norm = (p_rf / 255.0).clamp(0.0, 1.0);
                let p_g_norm = (p_gf / 255.0).clamp(0.0, 1.0);
                let p_b_norm = (p_bf / 255.0).clamp(0.0, 1.0);

                let p_white_accent = if p_r_norm >= 0.784 && p_g_norm >= 0.784 && p_b_norm >= 0.784
                {
                    1.0
                } else {
                    0.0
                };

                states.push(LogicalFixtureState {
                    intensity: p_intensity_norm,
                    max_power_factor,
                    color_rgb: (p_r_norm, p_g_norm, p_b_norm),
                    white_accent: p_white_accent,
                    flash_dimmer_factor,
                    white_flash_factor: white_factor,
                    is_in_strobe,
                    strobe_val,
                    color_wheel_dmx: None,
                    color_macro_dmx: None,
                    color_temp_dmx: None,
                    pan_norm: None,
                    tilt_norm: None,
                });
            }
            Some(states)
        } else {
            None
        };

        let buf = match self.universe_buffers.get_mut(&fixture.universe) {
            Some(b) => b,
            None => return,
        };

        let end_addr = (fixture.start_addr + fixture.profile.footprint).min(512);
        if fixture.start_addr < end_addr {
            encode_fixture_dmx(
                &fixture.profile,
                &state,
                pixel_states.as_deref(),
                effective_preference,
                &mut buf[fixture.start_addr..end_addr],
            );
        }
    }
}
