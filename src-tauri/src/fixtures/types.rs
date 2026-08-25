use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum NumberOrString {
    Number(f64),
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum StringOrBool {
    Bool(bool),
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum StringOrVecString {
    String(String),
    Vec(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportPlugin {
    pub plugin: String,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureMeta {
    pub authors: Vec<String>,
    pub create_date: String,
    pub last_modify_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_plugin: Option<ImportPlugin>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_page: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forum: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureBulb {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lumens: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureLens {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degrees_min_max: Option<(f64, f64)>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatrixPixelsPhysical {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<(f64, f64, f64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<(f64, f64, f64)>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixturePhysical {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<(f64, f64, f64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<f64>,
    #[serde(rename = "DMXconnector", skip_serializing_if = "Option::is_none")]
    pub dmx_connector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulb: Option<FixtureBulb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens: Option<FixtureLens>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_connectors: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix_pixels: Option<MatrixPixelsPhysical>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoboSlot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum GoboReference {
    Name(String),
    Slot(GoboSlot),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WheelResourceObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum WheelResource {
    String(String),
    Object(WheelResourceObject),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WheelSlot {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<WheelResource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gobo: Option<GoboReference>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureWheel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slots: Option<Vec<WheelSlot>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PixelGroupObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<StringOrVecString>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum PixelGroup {
    String(String),
    Vec(Vec<String>),
    Object(PixelGroupObject),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureMatrix {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_keys: Option<Vec<Vec<Vec<Option<String>>>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_groups: Option<BTreeMap<String, PixelGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_count: Option<(usize, usize, usize)>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmx_range: Option<(u32, u32)>,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_wanted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_click: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_channels: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound_controlled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound_sensitivity: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_angle: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_angle_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_angle_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_angle: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_angle_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_angle_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors_start: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors_end: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_temperature: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_temperature_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_temperature_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_number: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_number_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_number_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wheel: Option<StringOrVecString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shutter_effect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub random_timing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog_output: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog_output_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog_output_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_percent: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_percent_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_percent_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frost_intensity: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frost_intensity_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frost_intensity_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shake_speed: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shake_speed_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shake_speed_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shake_angle: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shake_angle_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shake_angle_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blade: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertion_start: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertion_end: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_shaking: Option<NumberOrString>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fine_channel_aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_value: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precedence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmx_value_resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<Capability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<Capability>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NamedChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fine_channel_aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_value: Option<NumberOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precedence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmx_value_resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<Capability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<Capability>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_for: Option<StringOrVecString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_channels: Option<Vec<Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ChannelRef {
    Key(String),
    Object(Box<NamedChannel>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Mode {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    pub channels: Vec<Option<ChannelRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical: Option<FixturePhysical>,
    #[serde(
        rename = "rdmPersonalityIndex",
        skip_serializing_if = "Option::is_none"
    )]
    pub rdm_personality_index: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureRdm {
    #[serde(rename = "modelId", skip_serializing_if = "Option::is_none")]
    pub model_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<FixtureMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<FixtureLinks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical: Option<FixturePhysical>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix: Option<FixtureMatrix>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wheels: Option<BTreeMap<String, FixtureWheel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_channels: Option<BTreeMap<String, FixtureChannel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_channels: Option<BTreeMap<String, FixtureChannel>>,
    pub modes: Vec<Mode>,
    pub fixture_key: String,
    pub manufacturer_key: String,
    #[serde(rename = "oflURL", skip_serializing_if = "Option::is_none")]
    pub ofl_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_wanted: Option<StringOrBool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdm: Option<FixtureRdm>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Manufacturer {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "rdmId", skip_serializing_if = "Option::is_none")]
    pub rdm_id: Option<u16>,
    pub fixtures: BTreeMap<String, Fixture>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawFixture {
    pub name: String,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub meta: Option<FixtureMeta>,
    #[serde(default)]
    pub links: Option<FixtureLinks>,
    #[serde(default)]
    pub physical: Option<FixturePhysical>,
    #[serde(default)]
    pub matrix: Option<FixtureMatrix>,
    #[serde(default)]
    pub wheels: Option<BTreeMap<String, FixtureWheel>>,
    #[serde(default)]
    pub available_channels: Option<BTreeMap<String, FixtureChannel>>,
    #[serde(default)]
    pub template_channels: Option<BTreeMap<String, FixtureChannel>>,
    #[serde(default)]
    pub modes: Vec<Mode>,
    #[serde(default)]
    pub fixture_key: Option<String>,
    #[serde(default)]
    pub manufacturer_key: Option<String>,
    #[serde(rename = "oflURL", default)]
    pub ofl_url: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub help_wanted: Option<StringOrBool>,
    #[serde(default)]
    pub rdm: Option<FixtureRdm>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ManufacturerParsed {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "rdmId", skip_serializing_if = "Option::is_none")]
    pub rdm_id: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureDownloadProgress {
    pub percentage: u8,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixtureLoadProgress {
    pub current: usize,
    pub total: usize,
    pub percentage: u8,
    pub manufacturer: String,
}

pub type Fixtures = BTreeMap<String, Manufacturer>;
