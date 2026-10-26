# Colors

Colors sets the colour of each group, the fade used when a colour changes, and
the colour effects. Only groups whose fixtures have colour channels appear on
this page.

## Colour table

Each row is a group; each column is a colour: white, CTB, CTO, red, orange,
yellow, lime, green, mint, cyan, azure, blue, violet, magenta and pink.

- The **large** button sets the group's primary colour. Clicking the active
  colour again clears it.
- The **small** button underneath sets the group's **secondary** colour, used by
  two-colour effects and blends.
- The header row repeats every colour with a down arrow. Clicking one applies
  that colour to **all** groups at once.

Colours are rendered according to each group's **Colour preference** on the
[Configure](/manual/setup/configure) page — colour mixing, colour wheel, CCT or
colour presets.

## Fade

The fade card shapes colour transitions:

- **Speed** — 0 to 3 seconds, in 0.25 s steps, defaulting to 1 s.
- **Order** — when several groups change colour at once, the fade can run
  `Left`, `Right`, `Wing` or `Center` across the group list. Clicking the active
  order turns it off.

## Effects

The colour effects table mirrors the intensity one:

- **Effect** — `Pulse`, `Chase`, `Ramp up`, `Ramp down`, `Sine` or `Rainbow`.
- **Alignment** — `Left`, `Right`, `Wing`, `Center`, `2 groups`, `3 groups`,
  `4 groups` or `Random`.
- **Bounce** — runs the animation back and forth.
- **Width** — 0 to 100 %, the active share of the cycle.
- **Reset** — clears the effect for that group.

The **BPM** knob above the table sets the colour-effect speed for every group,
from 0 to 240, defaulting to 60. It is independent from the intensity BPM, so
colour and intensity effects can run at different speeds.
