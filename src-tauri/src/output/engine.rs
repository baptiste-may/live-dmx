use std::collections::HashMap;
use std::time::Instant;

use crate::output::channels::{
    classify_channel, extract_fixture_channel_count, get_fixture_channel_capabilities,
    get_fixture_channel_name,
};
use crate::output::color_routing::{
    closest_color_position, effective_color_preference, map_temperature_to_dmx,
    rgb_to_color_temperature, ColorChannelSummary,
};
use crate::output::colors::{hsv_to_rgb, resolve_color_rgb};
use crate::output::effects::{
    calculate_effect_factor, calculate_effect_intensity, calculate_effective_fract,
    compute_random_strobe_active,
};
use crate::output::types::{
    ChannelType, FixtureColorTransition, FlashFadeTransition, GroupEffectState,
};
use crate::show::{ColorPreference, GroupSettings, PatchedFixture, ShowData};

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
    pub channels: Vec<ChannelType>,
    pub channel_capabilities: Vec<Vec<serde_json::Value>>,
    pub color_summary: ColorChannelSummary,
    pub has_dimmer: bool,
    pub treat_first_as_dimmer: bool,
    pub groups: Vec<u32>,
    pub primary_group_id: u32,
    pub group_positions: HashMap<u32, (usize, usize)>,
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
    cached_fixtures: Vec<CachedFixture>,
    cached_keys: Vec<FixtureCacheKey>,
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
    }

    pub fn reset_group_effect(&mut self, group_id: u32) {
        self.group_effects
            .insert(group_id, GroupEffectState::default());
    }

    pub fn reset_all(&mut self) {
        self.flash_states.clear();
        self.button_states.clear();
        self.group_effects.clear();
        self.group_color_effects.clear();
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

    fn ensure_fixtures_cache(&mut self, show_data: &ShowData) {
        let fixtures = &show_data.patch.fixtures;
        let is_cache_valid = self.cached_keys.len() == fixtures.len()
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
            let channel_count = extract_fixture_channel_count(f);
            let channels_to_write = channel_count.min(512usize.saturating_sub(start_addr));

            let mut has_dimmer = false;
            let mut has_color_channel = false;
            let mut classified_channels = Vec::with_capacity(channels_to_write);
            let mut channel_capabilities = Vec::with_capacity(channels_to_write);

            for ch in 0..channels_to_write {
                let ch_type = match get_fixture_channel_name(f, ch) {
                    Some(name) => classify_channel(&name),
                    None => {
                        if channels_to_write == 1 {
                            ChannelType::Dimmer
                        } else {
                            ChannelType::Other
                        }
                    }
                };
                channel_capabilities.push(get_fixture_channel_capabilities(f, ch));
                if ch_type == ChannelType::Dimmer {
                    has_dimmer = true;
                }
                if matches!(
                    ch_type,
                    ChannelType::Red
                        | ChannelType::Green
                        | ChannelType::Blue
                        | ChannelType::White
                        | ChannelType::Amber
                        | ChannelType::Cyan
                        | ChannelType::Magenta
                        | ChannelType::Yellow
                        | ChannelType::UV
                ) {
                    has_color_channel = true;
                }
                classified_channels.push(ch_type);
            }

            let treat_first_as_dimmer = !has_dimmer && !has_color_channel;
            let color_summary = ColorChannelSummary::from_channels(&classified_channels);

            new_cached.push(CachedFixture {
                id: f.id,
                universe: f.universe as u16,
                start_addr,
                channels: classified_channels,
                channel_capabilities,
                color_summary,
                has_dimmer,
                treat_first_as_dimmer,
                groups: f.groups.clone(),
                primary_group_id,
                group_positions,
            });

            new_keys.push(FixtureCacheKey {
                id: f.id,
                groups: f.groups.clone(),
                universe: f.universe,
                address: f.address,
                mode: f.mode.clone(),
            });
        }

        self.cached_fixtures = new_cached;
        self.cached_keys = new_keys;
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

            self.compute_fixture_dmx(
                fixture,
                local_index,
                group_count,
                &effect_state,
                color_local_index,
                color_group_count,
                &color_effect_state,
                elapsed,
                self.resolve_color_preference(fixture, &show_data.groups.settings),
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
        local_index: usize,
        group_count: usize,
        effect_state: &GroupEffectState,
        color_local_index: usize,
        color_group_count: usize,
        color_effect_state: &GroupEffectState,
        elapsed: f32,
        color_preference: ColorPreference,
    ) {
        if fixture.start_addr >= 512 {
            return;
        }

        let effective_preference =
            effective_color_preference(color_preference, fixture.color_summary);

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

        let group_master = self.get_group_master(fixture.primary_group_id) as f32 / 100.0;
        let global_master = self.get_group_master(0) as f32 / 100.0;
        let master_multiplier = group_master * global_master;

        let raw_intensity = calculate_effect_intensity(
            effect_state,
            elapsed,
            local_index,
            group_count,
            fixture.id,
            self.effect_bpm,
        );

        let intensity_level = ((raw_intensity as f32) * master_multiplier).clamp(0.0, 255.0) as u8;

        let strobe_val = if is_in_strobe_group {
            let freq = 1.0 + (self.strobe_rate as f32 / 100.0) * 19.0;
            if (elapsed * freq * 2.0 * std::f32::consts::PI).sin() > 0.0 {
                255
            } else {
                0
            }
        } else if is_in_random_strobe_group {
            if compute_random_strobe_active(fixture.id, elapsed, self.random_strobe_rate) {
                255
            } else {
                0
            }
        } else {
            0
        };

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

        let (r_f, g_f, b_f) = if color_effect_state.effect.as_deref() == Some("rainbow") {
            if let Some(fract) = calculate_effective_fract(
                color_effect_state,
                elapsed,
                color_local_index,
                color_group_count,
                fixture.id,
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
                color_local_index,
                color_group_count,
                fixture.id,
                self.color_effect_bpm,
            );

            (
                current_rgb_f.0 * (1.0 - color_factor) + secondary_rgb_f.0 * color_factor,
                current_rgb_f.1 * (1.0 - color_factor) + secondary_rgb_f.1 * color_factor,
                current_rgb_f.2 * (1.0 - color_factor) + secondary_rgb_f.2 * color_factor,
            )
        };

        let r = r_f.round().clamp(0.0, 255.0) as u8;
        let g = g_f.round().clamp(0.0, 255.0) as u8;
        let b = b_f.round().clamp(0.0, 255.0) as u8;

        let w = if r >= 200 && g >= 200 && b >= 200 {
            255
        } else {
            0
        };

        let buf = match self.universe_buffers.get_mut(&fixture.universe) {
            Some(b) => b,
            None => return,
        };

        let intensity_factor = (intensity_level as f32) / 255.0;

        let is_mix_routing = effective_preference == ColorPreference::ColorMix;
        let wheel_output = if effective_preference == ColorPreference::ColorWheel {
            fixture
                .channels
                .iter()
                .position(|&t| t == ChannelType::ColorWheel)
                .and_then(|idx| {
                    fixture
                        .channel_capabilities
                        .get(idx)
                        .map(|caps| closest_color_position(r, g, b, caps))
                })
        } else {
            None
        };
        let cct_output = if effective_preference == ColorPreference::Cct {
            fixture
                .channels
                .contains(&ChannelType::ColorTemperature)
                .then(|| map_temperature_to_dmx(rgb_to_color_temperature(r, g, b)))
        } else {
            None
        };
        let macro_output = if effective_preference == ColorPreference::ColorPresets {
            fixture
                .channels
                .iter()
                .position(|&t| t == ChannelType::ColorMacro)
                .and_then(|idx| {
                    fixture
                        .channel_capabilities
                        .get(idx)
                        .map(|caps| closest_color_position(r, g, b, caps))
                })
        } else {
            None
        };

        for (ch, &ch_type) in fixture.channels.iter().enumerate() {
            let ch_idx = fixture.start_addr + ch;

            if flash_dimmer_factor > 0.0 {
                let effective_dimmer = ((intensity_level as f32)
                    + (255.0 - intensity_level as f32) * flash_dimmer_factor)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let effective_intensity_factor = effective_dimmer as f32 / 255.0;

                match ch_type {
                    ChannelType::Dimmer => buf[ch_idx] = effective_dimmer,
                    ChannelType::Red
                    | ChannelType::Green
                    | ChannelType::Blue
                    | ChannelType::White
                    | ChannelType::Amber
                    | ChannelType::Cyan
                    | ChannelType::Magenta
                    | ChannelType::Yellow => {
                        if !is_mix_routing {
                            buf[ch_idx] = 0;
                            continue;
                        }
                        let raw = match ch_type {
                            ChannelType::Red => r as f32,
                            ChannelType::Green => g as f32,
                            ChannelType::Blue => b as f32,
                            ChannelType::White => w as f32,
                            ChannelType::Amber if r > 200 && g > 100 && b < 50 => 255.0,
                            ChannelType::Amber => 0.0,
                            ChannelType::Cyan if g > 200 && b > 200 && r < 50 => 255.0,
                            ChannelType::Cyan => 0.0,
                            ChannelType::Magenta if r > 200 && b > 200 && g < 50 => 255.0,
                            ChannelType::Magenta => 0.0,
                            ChannelType::Yellow if r > 200 && g > 200 && b < 50 => 255.0,
                            ChannelType::Yellow => 0.0,
                            _ => 0.0,
                        };
                        let base = if fixture.has_dimmer {
                            raw
                        } else {
                            raw * effective_intensity_factor
                        };
                        buf[ch_idx] = (base + (255.0 - base) * white_factor)
                            .round()
                            .clamp(0.0, 255.0) as u8;
                    }
                    ChannelType::UV => {
                        if !is_mix_routing {
                            buf[ch_idx] = 0;
                        } else {
                            buf[ch_idx] = (255.0 * white_factor).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                    ChannelType::ColorWheel => {
                        buf[ch_idx] = wheel_output.unwrap_or(0);
                    }
                    ChannelType::ColorMacro => {
                        buf[ch_idx] = macro_output.unwrap_or(0);
                    }
                    ChannelType::ColorTemperature => {
                        buf[ch_idx] = cct_output.unwrap_or(0);
                    }
                    ChannelType::Strobe => buf[ch_idx] = 0,
                    ChannelType::Other => {
                        if ch == 0 && fixture.treat_first_as_dimmer {
                            buf[ch_idx] = effective_dimmer;
                        } else {
                            buf[ch_idx] = 0;
                        }
                    }
                }
            } else if is_in_strobe_group || is_in_random_strobe_group {
                match ch_type {
                    ChannelType::Dimmer => buf[ch_idx] = strobe_val,
                    ChannelType::Red
                    | ChannelType::Green
                    | ChannelType::Blue
                    | ChannelType::White
                    | ChannelType::Amber
                    | ChannelType::Cyan
                    | ChannelType::Magenta
                    | ChannelType::Yellow
                    | ChannelType::UV => {
                        if !is_mix_routing {
                            buf[ch_idx] = 0;
                            continue;
                        }
                        let raw = match ch_type {
                            ChannelType::Red => r,
                            ChannelType::Green => g,
                            ChannelType::Blue => b,
                            ChannelType::White => w,
                            ChannelType::Amber if r > 200 && g > 100 && b < 50 => 255,
                            ChannelType::Amber => 0,
                            ChannelType::Cyan if g > 200 && b > 200 && r < 50 => 255,
                            ChannelType::Cyan => 0,
                            ChannelType::Magenta if r > 200 && b > 200 && g < 50 => 255,
                            ChannelType::Magenta => 0,
                            ChannelType::Yellow if r > 200 && g > 200 && b < 50 => 255,
                            ChannelType::Yellow => 0,
                            _ => 0,
                        };
                        buf[ch_idx] = if fixture.has_dimmer {
                            if strobe_val > 0 {
                                raw
                            } else {
                                0
                            }
                        } else {
                            ((raw as f32) * (strobe_val as f32 / 255.0)).clamp(0.0, 255.0) as u8
                        };
                    }
                    ChannelType::ColorWheel => {
                        buf[ch_idx] = wheel_output.unwrap_or(0);
                    }
                    ChannelType::ColorMacro => {
                        buf[ch_idx] = macro_output.unwrap_or(0);
                    }
                    ChannelType::ColorTemperature => {
                        buf[ch_idx] = cct_output.unwrap_or(0);
                    }
                    ChannelType::Strobe => buf[ch_idx] = strobe_val,
                    _ => {
                        if ch == 0 && fixture.treat_first_as_dimmer {
                            buf[ch_idx] = strobe_val;
                        } else {
                            buf[ch_idx] = 0;
                        }
                    }
                }
            } else {
                match ch_type {
                    ChannelType::Dimmer => buf[ch_idx] = intensity_level,
                    ChannelType::Red
                    | ChannelType::Green
                    | ChannelType::Blue
                    | ChannelType::White
                    | ChannelType::Amber
                    | ChannelType::Cyan
                    | ChannelType::Magenta
                    | ChannelType::Yellow
                    | ChannelType::UV => {
                        if !is_mix_routing {
                            buf[ch_idx] = 0;
                            continue;
                        }
                        let raw = match ch_type {
                            ChannelType::Red => r,
                            ChannelType::Green => g,
                            ChannelType::Blue => b,
                            ChannelType::White => w,
                            ChannelType::Amber if r > 200 && g > 100 && b < 50 => 255,
                            ChannelType::Amber => 0,
                            ChannelType::Cyan if g > 200 && b > 200 && r < 50 => 255,
                            ChannelType::Cyan => 0,
                            ChannelType::Magenta if r > 200 && b > 200 && g < 50 => 255,
                            ChannelType::Magenta => 0,
                            ChannelType::Yellow if r > 200 && g > 200 && b < 50 => 255,
                            ChannelType::Yellow => 0,
                            _ => 0,
                        };
                        buf[ch_idx] = if fixture.has_dimmer {
                            raw
                        } else {
                            ((raw as f32) * intensity_factor).clamp(0.0, 255.0) as u8
                        };
                    }
                    ChannelType::ColorWheel => {
                        buf[ch_idx] = wheel_output.unwrap_or(0);
                    }
                    ChannelType::ColorMacro => {
                        buf[ch_idx] = macro_output.unwrap_or(0);
                    }
                    ChannelType::ColorTemperature => {
                        buf[ch_idx] = cct_output.unwrap_or(0);
                    }
                    ChannelType::Strobe => {
                        buf[ch_idx] = 0;
                    }
                    ChannelType::Other => {
                        if ch == 0 && fixture.treat_first_as_dimmer {
                            buf[ch_idx] = intensity_level;
                        } else {
                            buf[ch_idx] = 0;
                        }
                    }
                }
            }
        }
    }
}
