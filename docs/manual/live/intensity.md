# Intensity

Intensity drives how bright the rig is: one master per group, four flash buttons
per group, and an effect row per group. The page is laid out with effects on the
left, masters underneath them, and flashes on the right.

## Masters

Every group has a vertical fader. The first one is the **Grand Master** (the
`All` group, id `0`): it scales every fixture in the show, including fixtures
that belong to no group. The other faders are the group masters.

A master is a multiplier over the group's computed intensity. It never changes
the patch, the colours or the effects; moving it back up restores the previous
look. The value of each fader is shown by its scale, in percent.

## Flashes

The flash table has one row per group and four momentary buttons:

| Button     | Effect                                                                |
| ---------- | --------------------------------------------------------------------- |
| **Simple** | Pushes the group to full intensity for as long as the button is held. |
| **White**  | Flashes the group to white.                                           |
| **Strobe** | Strobes the group.                                                    |
| **Random** | Strobes the group with randomised timing.                             |

Above the buttons, four knobs set the behaviour shared by all groups:

- **Fade** for **Simple** and **White** — 0 to 3 seconds, in 0.1 s steps. At `0`
  the flash is immediate.
- **Speed** for **Strobe** and **Random** — 0 to 100.

Buttons are momentary: the effect stops when the button is released.

## Effects

The effects table has one row per group (the `All` group is excluded) with:

- **Effect** — `Full`, `Pulse`, `Chase`, `Ramp up`, `Ramp down` or `Sine`.
  Clicking the active effect turns it off.
- **Alignment** — how the effect's phase is spread across the group:
  `Left`, `Right`, `Wing`, `Center`, `2 groups`, `3 groups`, `4 groups` or
  `Random`. Random regenerates a new permutation each time it is triggered.
- **Bounce** — makes the animation run back and forth instead of one way.
- **Width** — 0 to 100 %, the share of the cycle during which the effect is
  active. Defaults to 100 %.
- **Reset** — clears the effect, alignment and bounce for that group.

The **BPM** knob above the table sets the speed shared by all intensity effects,
from 0 to 240, defaulting to 60.

Effects and masters combine: the effect shapes the group's intensity, then the
group master scales the result, then the Grand Master scales everything.
