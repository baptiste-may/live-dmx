# Patch

The Patch page is where fixtures enter the show. It has three columns: the
fixture browser and patch form on the left, the group list and fixture table in
the middle, and the universe visualizer on the right.

## Find a fixture

The browser lists every fixture available offline plus the GDTF-Share catalog.

1. Type in **Search GDTF & generic fixtures...** — results filter by
   manufacturer as you type. `Esc` or **Clear** empties the search.
2. Open a manufacturer, then expand a fixture to see its **modes**. Each mode
   shows its channel count.
3. Hover a mode to preview the fixture: manufacturer, revision, power, beam
   angle, weight, DMX connector, dimensions and the full channel list. Modes
   that are not downloaded yet show a channel count and fetch their channel
   names from GDTF-Share when patched.

Badges in the fixture list tell you where a profile comes from:

| Badge       | Meaning                                                     |
| ----------- | ----------------------------------------------------------- |
| `Generic`   | Built-in fixture, available offline with no download.       |
| `Installed` | Manufacturer profile already cached on this machine.        |
| `GDTF`      | Profile available from GDTF-Share, downloaded when patched. |

The toolbar buttons next to the search field:

- The upload button imports a local `.gdtf` or `.zip` fixture file.
- The globe button opens the GDTF-Share settings. It is green when an account
  is connected.
- The refresh button rescans the local catalog.

Without a GDTF-Share account the browser only offers generic and already-cached
fixtures; **Connect GDTF-Share** opens the settings modal to sign in.

## Patch a fixture

1. Select a fixture mode in the browser. The selection card above the form shows
   the fixture, its manufacturer and the mode with its channel count.
2. Set **Universe** — the universe that will carry the fixture.
3. Set **Start Address** — the first DMX channel it occupies. The fixture
   occupies the next channels according to its mode.
4. Set **Count** to patch several identical fixtures at once. Addresses are
   allocated one after the other.
5. Give the fixture a **Name**. Names are what you see in the table, the
   visualizer and the live pages.
6. Click **Patch**.

## Groups

Groups are how the live pages address several fixtures at once. A fixture can
belong to more than one group.

- Type a name and click **Add Group** to create a group.
- Click a group chip to filter the fixture table to that group. The **All** group
  (id `0`) always shows every fixture and cannot be renamed or deleted.
- Select a group and use the pen button to rename it, or the trash button to
  delete it. Deleting a group asks for confirmation and does not delete its
  fixtures.

## Edit patched fixtures

The table lists ID, name, model, patch (`universe.address`, address padded to
three digits), and the groups each fixture belongs to. The chip above the table
shows which group is selected.

- Tick fixtures individually, or use the header checkbox to select every fixture
  currently displayed.
- The sliders button on a selected fixture (or on a row) opens the edit modal.
  The modal edits the selection in place:
  - **Fixture name** — rename one or many fixtures at once.
  - **Start address** — move the selection to a new address.
  - **Start universe** — move the selection to a new universe.
  - **Fixture groups** — toggle each group membership.
- The trash button deletes the selected fixtures after confirmation.

## Visualizer

The visualizer draws one universe as 512 channel cells. Pick a universe with the
stepper above the grid: every fixture patched on that universe is drawn over the
channels it occupies, with its id and name. Fixtures that span a row break
continue on the next row, so a long mode never looks truncated.
