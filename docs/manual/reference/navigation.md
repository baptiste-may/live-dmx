# Navigation

The navigation bar sits on the left of the application and is only shown while a
show is loaded. It contains the show actions, the two page sections, and the
application controls.

## Show title

The show name is displayed at the top of the bar.

- Click the pen button to edit it. The field is pre-filled with the current name.
- Press `Enter` or click the confirm button to apply the name; press `Esc` or
  click cancel to abandon the edit.
- An empty or whitespace-only name is refused: the previous name is kept.
- An unsaved show shows `(unsaved)` under its name.

## Show actions

| Action         | What it does                                                               |
| -------------- | -------------------------------------------------------------------------- |
| **New show**   | Opens an empty show named "My show", with no file on disk yet.             |
| **Open show**  | Opens a file picker for `.ldmx` and `.json` show files.                    |
| **Save show**  | Saves the show. Disabled when there is nothing to save and a file exists.  |
| **Undo**       | Reverts the last change. Disabled when there is nothing to undo.           |
| **Redo**       | Re-applies the last undone change. Disabled when there is nothing to redo. |
| **Close show** | Closes the show and returns to the welcome screen.                         |

**Save show** stays enabled while the show has never been saved, so the first
save can choose a location; afterwards it writes straight to that file. A
successful save shows a confirmation, a failed one an error.

If the show has unsaved changes, **New show**, **Open show** and **Close show**
open a confirmation first:

- **Cancel** — keep working, the action is abandoned.
- **Discard Changes** — run the action and lose the changes.
- **Save & Continue** — save, then run the action. If the save is cancelled, the
  dialog stays open and the action does not run.

Opening a show from the welcome screen lands on the Patch page by default.

## Sections

**Setup** — Input, Output, Patch, Configure. These prepare the show.

**Live** — Intensity, Colors. These drive the show.

The active page is highlighted. Each page keeps its own state while you move
between them.

## Bottom of the bar

- **Settings** opens the general settings modal.
- **Hide** collapses the bar to icons only. Category titles and labels disappear
  and separators take their place; hovering an icon shows its name in a tooltip.
  The show title and the `(unsaved)` marker are hidden while collapsed, and
  renaming the show is cancelled if you collapse the bar mid-edit.
- While collapsed, the same button becomes **Show sidebar** and expands the bar
  again.

## Keyboard shortcuts

Shortcuts are global while a show is loaded; they do nothing on the welcome
screen.

| Shortcut                                            | Action                        |
| --------------------------------------------------- | ----------------------------- |
| `Ctrl`/`Cmd` + `Z`                                  | Undo                          |
| `Ctrl`/`Cmd` + `Shift` + `Z`, or `Ctrl`/`Cmd` + `Y` | Redo                          |
| `Ctrl`/`Cmd` + `S`                                  | Save, when saving is possible |
