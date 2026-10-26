# User manual

Live DMX takes a show from an empty file to a lit stage in two passes: **Setup**
prepares the rig, **Live** drives it during the performance.

## Opening the application

The welcome screen shows the application name, its version, and an update button
when a newer release is available.

| Action          | What it does                                                                   |
| --------------- | ------------------------------------------------------------------------------ |
| New show        | Opens an empty show called "My show" with no file on disk yet.                 |
| Load show       | Opens a `.ldmx` show file (JSON show files are also accepted).                 |
| Import from MVR | Builds a show from an MVR scene file: layers, fixture instances and addresses. |
| Recents         | Reopens a recently used show, or removes it from the list.                     |
| Settings        | Opens the general settings modal.                                              |
| Exit            | Closes the application.                                                        |

Opening, creating or closing a show with unsaved changes asks whether to cancel,
discard the changes, or save before continuing.

Once a show is open, the navigation bar appears on the left and the show name is
displayed at its top. See [Navigation](/manual/reference/navigation) for the full
shell, its actions and the keyboard shortcuts.

## Setup

Prepare the rig once, before the doors open.

| Page                                 | What it covers                                                       |
| ------------------------------------ | -------------------------------------------------------------------- |
| [Patch](/manual/setup/patch)         | Find fixtures, address them, group them, and visualize a universe.   |
| [Configure](/manual/setup/configure) | Per-group limits: power, pan and tilt range, colour and pixel modes. |
| [Input](/manual/setup/input)         | External control sources.                                            |
| [Output](/manual/setup/output)       | Route universes to a protocol or a USB interface, and monitor DMX.   |

A typical order is: patch the fixtures, group them, configure each group, then
route the universes they use.

## Live

Everything you touch during the show.

| Page                                | What it covers                                      |
| ----------------------------------- | --------------------------------------------------- |
| [Intensity](/manual/live/intensity) | Group masters, flash buttons and intensity effects. |
| [Colors](/manual/live/colors)       | Colour table, colour fade and colour effects.       |

## Reference

| Page                                       | What it covers                            |
| ------------------------------------------ | ----------------------------------------- |
| [Navigation](/manual/reference/navigation) | The sidebar, the show actions, shortcuts. |
| [Glossary](/manual/reference/glossary)     | DMX and lighting vocabulary.              |
