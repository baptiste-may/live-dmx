# Configure

Configure sets per-group behaviour that applies everywhere in the show: the
power ceiling, the allowed pan and tilt range, and how colours and pixels are
driven.

## Select a group

The **Selection** list contains **All** and every group created on the Patch
page. The **Settings** card always shows which group is being edited in its
header badge. Changes apply to the selected group only.

## Colour preference

Controls how the group's colour is rendered, based on what the fixtures support:

| Value           | Behaviour                                                        |
| --------------- | ---------------------------------------------------------------- |
| `Color Mix`     | Mixes the fixture's colour channels (RGB, RGBW, CMY, ...).       |
| `Color Wheel`   | Uses the fixture's colour wheel.                                 |
| `CCT`           | Drives the colour temperature channel from the requested colour. |
| `Color presets` | Uses the fixture's predefined colours (colour macro channel).    |

When the selected group has no colour channels, the selector shows `None` and is
disabled.

## Pixel preference

Controls how multi-cell fixtures (LED bars, battens, strips) run effects:

| Value        | Behaviour                                                                                       |
| ------------ | ----------------------------------------------------------------------------------------------- |
| `Single`     | One step per fixture; every pixel on a fixture shows the same colour and intensity.             |
| `Individual` | One step per pixel; effects run across the whole group as a single sequence.                    |
| `Grouped`    | Splits each fixture into **Group count** zones; effects progress zone by zone across the group. |

`Group count` appears when `Grouped` is selected, with a minimum of 2 and a
maximum equal to the largest pixel count in the group. Groups without pixels
show `None`.

## Pan and tilt

The square grid represents the fixture's physical range.

- The horizontal slider sets the **pan** range; the vertical slider sets the
  **tilt** range. The limits of each slider come from the fixtures themselves.
- The numeric fields repeat the same values. A minimum can never move past the
  maximum, and values are clamped to the fixtures' physical bounds.
- **Inverse Pan** and **Inverse Tilt** reverse the direction of movement for the
  group.
- The bounds are hidden or disabled when the group has no pan or no tilt
  channels.

## Max Power

The **Max Power** fader limits the group from 0 to 100 %. It is a ceiling, not a
multiplier: a group limited to 50 % never exceeds half intensity, even with its
master at full and even while flashing or strobing. The limit is applied to the
computed output, including per-pixel intensities.

## Layout

The **Layout** panel on the right is a placeholder reserved for a stage layout
preview; it holds no controls yet.
