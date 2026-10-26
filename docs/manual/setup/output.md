# Output

Output routes each universe to a destination and shows what is actually being
sent. The page has three parts: the universe list, the physical interfaces
detected on the machine, and the DMX monitor.

## Universes

Each universe is a row with a status light, its number, a target selector, an
enable toggle and a delete button.

- **Add Universe** appends a universe; **Add First Universe** appears when the
  list is empty.
- The target selector chooses where the universe goes:

  | Target               | Transport                          |
  | -------------------- | ---------------------------------- |
  | `None`               | Computed but not sent anywhere.    |
  | `ArtNet`             | Art-Net, UDP port 6454.            |
  | `sCan (sACN)`        | Streaming ACN (E1.31) multicast.   |
  | `KiNET`              | Philips Color Kinetics.            |
  | `OPC`                | Open Pixel Control, TCP port 7890. |
  | `OSC`                | Open Sound Control messages.       |
  | A detected interface | The USB DMX interface, by name.    |

- The toggle mutes or resumes the universe without losing its target.
- The trash button removes the universe. The last remaining universe cannot be
  removed.

The status light summarises the route:

| Status       | Meaning                                                     |
| ------------ | ----------------------------------------------------------- |
| Sending data | Enabled and routed to a working target.                     |
| Idle         | Enabled but no target selected (`None`).                    |
| Error        | The selected physical interface is missing or disconnected. |
| Disabled     | The universe is muted.                                      |

The counters next to the title count enabled universes with an error (missing or
disconnected interface) and with a warning (no target, or a busy interface).

## Physical interfaces

Every USB DMX interface found on the machine is listed with its name, device
path, port, and measured frame rate. A badge shows whether it is **Connected**,
**Busy** (already driven by another universe or process) or **Disconnected**;
the footer line lists the universes currently routed to it, or says it is not
assigned.

Use the rescan button (circular arrow) after plugging an interface in or out. With
no interface detected, **Rescan Devices** appears in the empty state.

## DMX Output

The **DMX Output** monitor at the bottom refreshes every 100 ms and draws one
square card per universe: 512 cells in 32 × 16, each cell showing that channel's
live value in its tooltip and its intensity as opacity. The pause button freezes
the display so a value can be read without it moving; the data keeps flowing to
the fixtures.
