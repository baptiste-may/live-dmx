# Rendering Pipeline (Profiles, Channels, Colours, Effects)

Read this when changing GDTF mode compilation, channel classification and
defaults, pan/tilt/shutter heuristics, pixel detection, `encode_fixture_dmx`,
colour mixing, colour routing/transitions, effects, alignments, or pixel
preferences. The surrounding engine, masters, flashes and strobes are documented
in `docs/contributing/dmx-output.md`.

## Module map (src-tauri/src/output/)

| File               | Content                                                                                                            |
| ------------------ | ------------------------------------------------------------------------------------------------------------------ |
| `profile.rs`       | `LogicalAttribute`, `ChannelResolution`, `CompiledFixtureProfile`, `parse_channel_name`, `compile_fixture_profile` |
| `pipeline.rs`      | `LogicalFixtureState`, `encode_fixture_dmx`, `max_power` clamping                                                  |
| `channels.rs`      | `classify_channel` (`ChannelType`), name/default/shutter/capability extraction                                     |
| `colors.rs`        | `resolve_color_rgb`, `hsv_to_rgb`                                                                                  |
| `color_routing.rs` | `ColorChannelSummary`, `effective_color_preference`, `closest_color_position`, CCT conversion                      |
| `effects.rs`       | SplitMix64, Fisher-Yates permutations, effect factors, random strobe                                               |
| `engine.rs`        | Group colours, transitions, active effect selection, pixel steps                                                   |

The pipeline is **pure**: it receives a normalized `LogicalFixtureState` and a
compiled profile, and writes DMX bytes into a slice. It never reads `ShowData`,
the clock, or a lock, which keeps it testable.

```text
patched mode / EmbeddedGdtfMode
        │ compile_fixture_profile
        ▼
CompiledFixtureProfile  ──┐
                          │  encode_fixture_dmx(profile, state, pixel_states, preference, buf)
LogicalFixtureState    ───┘
        │
        ▼
DMX bytes of the footprint
```

## Two classifications, never to be confused

|            | `LogicalAttribute` (`profile.rs`) | `ChannelType` (`channels.rs`)    |
| ---------- | --------------------------------- | -------------------------------- |
| Used for   | DMX encoding                      | Configure capabilities / options |
| Fine       | separate `is_fine` flag           | `PanFine`, `TiltFine` variants   |
| `duration` | treated as curve/mode             | not explicitly excluded          |
| Result     | 26 variants including `Shutter`   | 23 variants including `Strobe`   |

`parse_channel_name` and `classify_channel` share the base logic (lowercase,
substring match, `colorAdd_`/`colorSub_` prefixes) but are not identical. A new
naming rule that also affects Configure must be applied to both.

## Compiling a profile

`compile_fixture_profile(channel_names, channel_defaults, shutter_open_values,
pan_bounds, tilt_bounds, capabilities)` runs several passes.

### 1. Raw classification

Names are lowercased and classified by substring, in this notable order:

1. `pan` → Pan; `tilt` → Tilt; `xyz` → PositionXYZ.
2. `fine`, `lsb`, `16 bit`, `16bit`, `16-bit` → fine flag (except
   pan/tilt/dimmer/colour, which keep their attribute).
3. `beam` + `dimmer`/`intensity` → BeamDimmer; `beam` + `white` → BeamWhite.
4. `dimmer`/`intensity` → Dimmer, unless the name contains `curve`, `mode`,
   `speed`, `time` or `duration`.
5. Colours: `lime`/`green-yellow`/`coloradd_gy` → Lime **before** Green;
   `red`/`r`/`coloradd_r` → Red; etc. `colorSub_C/M/Y` → Cyan/Magenta/Yellow.
6. `strobe`/`shutter` → Shutter (except curve/mode names).
7. `wheel` → ColorWheel; `macro`/`preset`/`colors`/`color slot` → ColorMacro;
   `cct`/`cto`/`ctb`/`ctc`/`color temperature` → ColorTemperature.
8. `gobo`, `prism`, `zoom`, `focus`, `control`/`reset`, otherwise Other.

If an explicit shutter-open value exists for the index and the attribute is
Other, it becomes Shutter.

### 2. Fallback heuristics

| Situation                           | Rule                                                                                             |
| ----------------------------------- | ------------------------------------------------------------------------------------------------ |
| Batten/tilt bar without pan/tilt    | `footprint ≥ 4`, channel 0 = Other, channel 1 fine → Tilt coarse + Tilt fine                     |
| Placeholder before the first dimmer | name `nofeature`, `channel N`, `dummy...` or `raw...` → Shutter                                  |
| Multi-dimmer hybrid + RGB           | a dimmer whose section up to the next dimmer contains no RGB → BeamDimmer (and its fine partner) |
| Isolated white in a hybrid          | once a BeamDimmer exists, all White channels become BeamWhite                                    |

These cover incomplete or badly named GDTF modes (GLP X4 Bar, JDC1/JDC2 IP,
Color STRIKE M, etc.) and are generic: no fixture name is hardcoded.

### 3. Defaults

`default_dmx` is filled for the **whole footprint**:

- explicit `channel_defaults` value when it is > 0;
- otherwise 128 for Pan/Tilt coarse;
- otherwise 255 for PositionXYZ;
- otherwise 0.

An explicit default of 0 is therefore ignored in favour of the attribute rule,
which centres pan/tilt even when a GDTF leaves 0.

### 4. 16-bit pairs

A coarse channel is paired with the **immediately following** channel when that
channel is fine **and** has the same attribute → `Bits16 { coarse, fine }`. An
unpaired fine channel stays a `Bits8` at its own offset and does not consume the
next channel.

### 5. Shutter opening

For each Shutter channel:

1. explicit value at index `i`;
2. otherwise the value at `i + 1` when the next channel is not itself a Shutter
   (tolerates 1-based indexing);
3. a value ≤ 15 is replaced by **255** (closed/LED-off ranges);
4. otherwise the value is kept;
5. with no explicit value: `default_dmx[i]` when > 15, else 255.

### 6. Flags

- `has_dimmer`: at least one Dimmer coarse channel.
- `treat_first_as_dimmer`: only for **single-channel** profiles without Dimmer
  or PositionXYZ. Disabled as soon as a position channel exists, so a DMX mover
  is not modulated by intensity.
- `is_cmy`: at least one Cyan/Magenta/Yellow **and no** Red/Green/Blue.
- `color_summary`: `has_mix`, `has_wheel`, `has_macro`, `has_temp`, used by
  colour routing.
- `channel_capabilities`: copied per channel, or empty vectors.

### 7. Pixel detection

| Case                        | Detection                              | Result                                                                               |
| --------------------------- | -------------------------------------- | ------------------------------------------------------------------------------------ |
| Multi-cell colour           | Several Red channels (or Cyan for CMY) | One pixel per occurrence; colour channels between two primaries belong to that pixel |
| Per-pixel dimmer            | coarse dimmers count = pixel count     | the dimmer is attached to its pixel                                                  |
| Multi-dimmer without colour | several dimmers, no colour primary     | one pixel per dimmer (blinders, sunstrips)                                           |
| Otherwise                   | —                                      | a single pixel (index 0) with colours + dimmer                                       |

`pixel_count` is at least 1. Each `CompiledChannel` carries an optional
`pixel_index`; `pixels` lists the channel indexes of each cell.

## Where the compiled mode comes from

`LiveEngine::build_cached_fixture` first looks for an embedded definition:

1. direct key `<manufacturer> <type>` in lowercase, or
   `<manufacturerKey>@<fixtureKey>`;
2. otherwise a search by type name, `gdtfSpec` or `fixtureKey` (keys sorted for
   determinism);
3. when a mode with the same name exists, it is **merged** with the patched
   mode's `channelDefaults` and `shutterOpenValues` (the patched values win),
   then compiled with the capabilities;
4. otherwise `compile_from_patched_fixture` compiles the patched mode JSON
   directly. It accepts channels as strings or objects (`name`, or `channel`,
   plus `capability`/`capabilities`) and reads `channelDefaults`,
   `shutterOpenValues`, `panBounds`, `tiltBounds`.

## Encoding (`encode_fixture_dmx`)

### Initialization

`buf[..footprint]` is first filled with `profile.default_dmx`. This immediately
covers Gobo, Prism, Zoom, Focus, Control and every undriven channel.

### Intensity and dimmer

- 8-bit `Intensity`/Dimmer: `round(level * 255)`; 16-bit:
  `round(level * 65535)`, coarse `>> 8`, fine `& 0xFF`.
- A channel with a `pixel_index` takes the pixel's level.
- When pixel states exist and the dimmer is a **master** (no per-cell dimmer),
  it is forced to 1.0 as soon as any pixel is active, flashing or strobing,
  otherwise 0. The intensity is then carried by the pixel colour channels.
- Otherwise the master dimmer uses the fixture's effective intensity.
- `effective_dimmer` applies flash, strobe and the `max_power_factor` ceiling
  (see `docs/contributing/dmx-output.md`).

### Shutter

- Strobe active: open value when `strobe_val > 0`, else 0.
- Otherwise: open when the effective intensity > 0 or a flash is active;
  otherwise `default_dmx`.
- `BeamDimmer` and `BeamWhite` are **always 0**: hybrid white tubes do not light
  with the wash.

### Colour

Colour channels are routed by the effective preference:

- outside `Color Mix`, all RGB/RGBW/CMY/UV mix channels are set to **0** (wheel,
  macro or CCT takes over);
- in `Color Mix`, values depend on the attribute: direct R/G/B, White via
  `white_accent`, Amber when red+green without blue, Lime **always 0**, CMY
  subtractive when `is_cmy` (1 - component) and additive accents otherwise, UV
  driven by the white flash;
- `color_gate` prevents double dimming: open when a dedicated dimmer carries the
  intensity, otherwise equal to the effective intensity; in strobe it is open
  (or capped by max power when there is no dedicated dimmer);
- 16-bit colour channels encode coarse/fine like the dimmer; the gate is applied
  to the normalized value before conversion.

### Wheel, macro, temperature

- `ColorWheel`: precomputed `color_wheel_dmx`, else 0.
- `ColorMacro`: `color_macro_dmx`, else 0.
- `ColorTemperature`: `color_temp_dmx`, else 0.

These values are computed upstream only when the effective preference matches
and the channel exists.

### Pan, tilt, position

- Pan/Tilt are written only when `pan_norm`/`tilt_norm` are `Some`; otherwise the
  profile defaults remain (centre 128 coarse, 0 fine).
- PositionXYZ always writes **255** and is never modulated by intensity.
- An `Other` channel at offset 0 is driven by intensity only when
  `treat_first_as_dimmer` is true.

## Colour resolution

`resolve_color_rgb` accepts:

- `#RRGGBB` (invalid values are ignored);
- the names `white`, `ctb`, `cto`, `red`, `orange`, `yellow`, `lime`, `green`,
  `mint`, `cyan`, `azure`, `blue`, `violet`, `magenta`, `pink`;
- anything else → **white** (255, 255, 255).

`ctb` = (198, 230, 255), `cto` = (255, 199, 138).

A fixture's selected colour is the first colour defined by its groups, then the
group 0 colour, then `white`. The secondary colour follows the same lookup but
**without** a white fallback: absent means black (0, 0, 0).

### Transitions

`set_group_color(group_id, color, show)`:

- updates `group_colors`;
- when `color` is `None` or `color_fade_speed <= 0`, removes the transition and
  writes the target immediately into `fixture_last_colors`;
- otherwise creates a `FixtureColorTransition` per fixture of the group (group 0
  = all fixtures).

Each transition stores `start_rgb` (the actual current colour, including an
in-progress fade), `target_rgb`, `delay_secs` and `duration_secs`.

| `color_fade_order` | Phase `p_i` (0 = first)                                    |
| ------------------ | ---------------------------------------------------------- |
| `None`             | all simultaneous, duration = `color_fade_speed`            |
| `left`             | `i / (n - 1)`                                              |
| `right`            | `(n - 1 - i) / (n - 1)`                                    |
| `wing`             | `abs((i + 0.5)/n - 0.5) * 2` (0 at centre, 1 at edges)     |
| `center`           | `1 - abs((i + 0.5)/n - 0.5) * 2` (1 at centre, 0 at edges) |

For a given order: `individual_duration = max(speed * 0.5, 0.05)` and
`max_delay = max(speed - individual_duration, 0)`, then `delay = p_i *
max_delay`. Fixtures therefore do not all change at once, but each transition is
short and overlaps its neighbours.

At each tick `get_fixture_current_rgb_f32` applies: before the delay → start
colour; after delay + duration, or duration ≤ 0 → target; in between → linear
RGB interpolation. `set_all_groups_color` writes the colour into **all** groups
(and 0), then starts the transition on group 0; `None` resets everything to
white. Finished transitions are purged on the next tick, while
`fixture_last_colors` keeps the last colour actually emitted.

## Colour preference and routing

`GroupSettings.color_preference`: `Color Mix` (default), `Color Wheel`, `CCT`,
`Color presets`. A fixture's preference is the first one found in its group
order, then group 0; `Color Mix` by default.

`effective_color_preference` corrects an impossible preference against the
channels actually present (`ColorChannelSummary`):

| Requested      | Fallback chain                       |
| -------------- | ------------------------------------ |
| `ColorMix`     | mix → wheel → macro → temp → mix     |
| `ColorWheel`   | wheel → mix → macro → temp → wheel   |
| `ColorPresets` | macro → mix → wheel → temp → presets |
| `Cct`          | temp → mix → wheel → macro → cct     |

With no colour channel at all, the requested preference is kept and nothing is
emitted.

### Wheel and presets

`closest_color_position(r, g, b, capabilities)`:

- empty capabilities → fixed palette of 8 slots (white, red, orange, yellow,
  green, cyan, blue, magenta), position = middle of the slot;
- otherwise, for each capability with hex colours (`color` or `colors`) and
  `dmxFrom`/`dmxTo`, position = `(from + to) / 2`, chosen by minimal Euclidean
  RGB distance;
- malformed capabilities are ignored; if none is usable, fall back to the
  palette.

### Colour temperature

`rgb_to_color_temperature`: linearized sRGB → XYZ (sRGB matrix) → xy
chromaticity → McCamy cubic formula, clamped to **2000–10000 K**.
`map_temperature_to_dmx` projects that range linearly onto 0–255. A zero XYZ sum
returns 3200 K. Wheel/macro/CCT channels are computed only when the effective
preference matches **and** the channel exists; otherwise they stay 0 and the mix
channels are off.

## Additive and subtractive mixing

In `Color Mix`:

| Channel                              | Value                                                                           |
| ------------------------------------ | ------------------------------------------------------------------------------- |
| Red / Green / Blue                   | current RGB component                                                           |
| White                                | `white_accent` = 1 when R, G and B ≥ 0.784, else 0                              |
| Amber                                | 1 when R > 0.78, G > 0.39 and B < 0.20                                          |
| Lime                                 | **always 0** (white-point neutrality, never pulls to yellow)                    |
| Cyan / Magenta / Yellow, CMY fixture | `1 - R`, `1 - G`, `1 - B` (subtractive)                                         |
| Cyan / Magenta / Yellow, RGB fixture | additive accents when the colour is highly saturated (thresholds ≈ 0.78 / 0.20) |
| UV                                   | white flash factor                                                              |

A CMY fixture has no RGB channels: `is_cmy` is true and a white opening pushes
all three CMY channels to 0. An RGB fixture that also has CMY channels treats
them as **accents**, not as the main mix. White is never obtained by pushing
Lime: it stays at 0.

**Known gap:** the additive `Yellow` branch in `pipeline.rs` tests `r > 0.78 &&
b > 0.78 && b < 0.20` (blue twice), so it can never fire. Treat additive Yellow
as unavailable until that guard is fixed and tested.

### Colour gate

The gate avoids double dimming:

- no dedicated dimmer → the gate carries the effective intensity (capped by max
  power);
- dedicated dimmer (master or cell) → gate open at 1.0;
- strobe → gate open on the on-phase, else 0; without a dedicated dimmer the
  on-phase is capped by max power.

## Effects

`GroupEffectState { effect, alignment, bounce, width }`, with `width` defaulting
to 100.

The active intensity effect is the **primary group's** effect when defined,
otherwise group 0's. Colour follows the same rule with a **distinct** active
group. The active group also determines the fixture's position and total steps
for the phase.

### Waveform

`calculate_effective_fract` returns a 0–1 cycle fraction, or `None` when the
effect is `none`/`off` or `bpm == 0`:

```text
speed = bpm / 60 * 2π
norm_t = (elapsed * speed + phase_offset) / 2π
fract = fract(norm_t)
if bounce and odd cycle: fract = 1 - fract
```

`calculate_effect_factor` then applies the width:

```text
if width <= 0 or fract >= width: 0
active = fract / width
t = active * 2π
```

| Effect         | Value                         |
| -------------- | ----------------------------- |
| `full`         | 1.0 (ignores width)           |
| `pulse`        | `max(sin(t), 0)^2.5`          |
| `chase`        | 1 when `sin(t) > 0.5`, else 0 |
| `ramp_up`      | `active`                      |
| `ramp_down`    | `1 - active`                  |
| `sine`         | `(sin(t) + 1) / 2`            |
| any other name | 1.0                           |

`calculate_effect_intensity = round(factor * 255)`.

### Alignment / phase

| Alignment                | Phase offset                                         |
| ------------------------ | ---------------------------------------------------- |
| `single`, `none`, absent | 0                                                    |
| `left`                   | `i / n * 2π`                                         |
| `right`                  | `(1 - i/n) * 2π`                                     |
| `wing`                   | distance to centre × `2π`                            |
| `center`                 | `(1 - distance to centre) * 2π`                      |
| `two`                    | `(i % 2) * π`                                        |
| `three`                  | `(i % 3) * 2π/3`                                     |
| `four`                   | `(i % 4) * π/2`                                      |
| `rng`                    | `i / n * 2π`, where `i` is already the permuted slot |

`rng` is not an independent draw per fixture: the engine replaces the index with
a unique slot of the group permutation, then applies the same formula as `left`.
That is what guarantees evenly spaced phases.

### Colour effects

Two families:

- **rainbow**: when `fract < width`, colour = `hsv_to_rgb(fract / width, 1, 1)`;
  otherwise the current colour. If `width <= 0`, the current colour is kept.
- **other effects**: linear interpolation between the current colour and the
  secondary colour using `calculate_effect_factor`. Without a secondary colour,
  the target is black.

## Pixel preferences

`GroupSettings.pixel_preference` and `pixel_group_count` decide how many
**steps** each fixture adds to the group's global cycle. Steps are computed once
in `ensure_fixtures_cache`, in patch order.

| Preference   | Steps per fixture                  | Effect                                             |
| ------------ | ---------------------------------- | -------------------------------------------------- |
| `Single`     | 1                                  | all pixels of a fixture share colour and intensity |
| `Individual` | `P_i` (pixel count)                | each pixel is a step; group total = `Σ P_i`        |
| `Grouped`    | `clamp(pixel_group_count, 1, P_i)` | splits the fixture into contiguous zones           |

For a group: `total = Σ steps_i`, `offset_i = Σ steps of previous fixtures`.
Group 0 contains all fixtures; an explicit group contains its members. A fixture
in several groups has a position and step count **per group**.

### Per-pixel attribution

For pixel `p` of a fixture with `P` pixels:

```text
raw_step = offset + p * fixture_steps / P
```

The step is then mapped through the RNG permutation when the alignment is `rng`,
then evaluated over `total` steps. Effects are therefore **global to the group**,
never reset per fixture. Pixel states are produced only when
`profile.pixel_count > 1` **and** at least one step count (intensity or colour)
is > 1. In `Single`, one master state is used and pixels stay identical.

### Master dimmer and pixels

- A **master** dimmer without per-cell dimmers opens at 1.0 as soon as a pixel is
  active, flashing or strobing; intensity is carried by the pixel colour
  channels.
- A **per-cell** dimmer modulates each cell individually.
- A fixture with no dimmer at all uses the colour gate as its intensity path.

## RNG: permutation and seed

- Initial seed: `0x123456789abcdef0`.
- Selecting `rng` on an effect **regenerates** the seed (`splitmix64`) and
  clears the group's permutation; the colour permutation uses `seed + 1`.
- `generate_shuffled_steps(count, seed)`: Fisher-Yates with SplitMix64, up to
  **64** attempts.
- Each attempt is rejected when `is_too_structured` detects a strictly
  increasing/decreasing run, a cyclic sweep, a V / inverted V (wing/center), an
  interleaved pattern, a symmetric mirror, or a long consecutive run.
- After 64 failures, the last permutation is used as-is.
- `get_rng_step` memoizes the permutation per group and regenerates it when the
  step count changes.

Properties: every slot 0..N-1 appears **exactly once**, no duplicate, no gap. The
permutation is deterministic for a given seed and N.

## Invariants and pitfalls

- Do not confuse the **footprint** (mode channel count) with the buffer size:
  encoding is bounded by `min(footprint, buf.len())`. A mode wider than the
  remaining slice is truncated.
- A mode with no channel names yields a footprint-0 profile: nothing is encoded.
- `default_dmx` is the base: an unhandled channel keeps its default, it is not
  reset to 0.
- An explicit default of 0 never overrides pan/tilt centring or PositionXYZ 255.
- `treat_first_as_dimmer` applies only to **single-channel** profiles.
- The heuristics depend on channel **order**: moving a channel can change its
  classification.
- A fine channel immediately after a coarse channel of the **same attribute** is
  absorbed; otherwise it stays independent.
- Configure's `ChannelType` and rendering's `LogicalAttribute` must stay
  coherent.
- `color_fade_speed <= 0` or a `None` colour means no transition, applied
  immediately.
- A fade order with `n <= 1` sets the phase to 0.
- An absent secondary colour is **black**, not white.
- A colour effect with no active effect (`none`/`off`) or BPM 0 yields a 0
  factor (current colour, no rainbow).
- `full` ignores width; the other effects are null outside the width.
- `chase` is not a 50 % square: the threshold is `sin(t) > 0.5`.
- Pixel steps are **global to the group**; never reset the effect per fixture.
- `Grouped` is clamped to a minimum of **1** here (the Configure UI offers 2):
  a value of 0 or 1 gives a single zone.
- RNG must remain a permutation: never replace it with an independent draw.
- Lime stays 0, even in white: that is deliberate for the white point.
- Do not duplicate preference resolution in the frontend:
  `effective_color_preference` is the reference.
- The pipeline must never read `ShowData` or the clock; it stays testable with a
  pure state.

## Tests

- `tests/output/channels.rs`: classification, capabilities, name/default/shutter
  extraction.
- `src/output/profile.rs` (unit tests): 16-bit mover, CMY, RGB without dimmer,
  batten heuristic.
- `src/output/pipeline.rs` (unit tests): 16-bit dimmer, shutter, strobe, max
  power ceiling, NaN, colour gate, CMY, auxiliary defaults, pixels.
- `tests/output/colors.rs`: group colours, `set_all_groups_color`, names/hex,
  simultaneous/left/wing/center fades, `hsv_to_rgb`, concurrent fades, retargeting
  mid-fade.
- `tests/output/color_routing.rs`: channel summary, fallback chains, slots with
  and without capabilities, malformed data, known/clamped CCT, DMX mapping.
- `tests/output/effects.rs`: white/simple flash and fade, BPM, width, effect
  types, random strobe, centre, permutation, structured-pattern rejection,
  forced random.
- `tests/output/engine.rs`: effects and secondary colours, rainbow, `Single`/
  `Individual`/`Grouped` pixel preferences, RNG phase and offset.
- `tests/output/fixtures_integration.rs`: Mac Aura PXL, Robe iBolt, GLP X4 Bar
  20, JDC2 IP, DMX mover, Chauvet Epix Strip, Gigapix Wash, Cameo Opus SP5P,
  Ayrton EagleStrike (1-indexed shutter).
