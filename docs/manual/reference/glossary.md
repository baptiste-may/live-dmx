# Glossary

| Term                    | Definition                                                                                                                                                           |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Alignment**           | Distribution of an effect's phase across the fixtures or pixels of a group: Left, Right, Wing, Center, 2 groups, 3 groups, 4 groups or Random.                       |
| **All group**           | Special group (id `0`) containing every fixture. Its master is the Grand Master.                                                                                     |
| **Art-Net**             | Protocol sending DMX over UDP (port 6454).                                                                                                                           |
| **Bounce**              | Effect option that runs the animation back and forth instead of one way.                                                                                             |
| **BPM**                 | Beats per minute: the speed of intensity or colour effects (0 to 240, default 60).                                                                                   |
| **CCT**                 | Correlated Colour Temperature, expressed in kelvins. A colour mode that drives the fixture's colour-temperature channel from the requested colour.                   |
| **Channel**             | DMX slot in a universe (1 to 512). Each channel carries a value from 0 to 255.                                                                                       |
| **Color Mix**           | Colour mode that mixes the fixture's colour channels (RGB, RGBW, CMY, ...).                                                                                          |
| **Color preference**    | How a group's colour is rendered: Color Mix, Color Wheel, CCT or Color presets.                                                                                      |
| **Color presets**       | Colour mode that uses the fixture's predefined colours (colour macro channel).                                                                                       |
| **Color Wheel**         | Colour mode that uses the fixture's colour filter wheel.                                                                                                             |
| **DMX**                 | Digital Multiplex: the standard for controlling lighting through digital channels (512 channels per universe, values 0 to 255).                                      |
| **DMX address**         | Position of the first channel a fixture occupies in a universe. The start address is entered during patch (1 to 512).                                                |
| **DMX monitor**         | Real-time display of the 512 channel values of a universe.                                                                                                           |
| **DMX value**           | Numeric level of a channel, from 0 to 255. 0 means off or closed, 255 maximum or open.                                                                               |
| **Dimmer**              | Channel or attribute that sets the light intensity of a fixture.                                                                                                     |
| **Effect**              | Animation applied to a group: Full, Pulse, Chase, Ramp up, Ramp down, Sine (intensity) or Rainbow (colour).                                                          |
| **Embedded definition** | GDTF profile stored inside the show file so the show stays portable and works offline.                                                                               |
| **Fade**                | Fade duration of an intensity or colour change (0 to 3 seconds).                                                                                                     |
| **Fixture**             | Luminaire or DMX-controlled device: PAR, moving head, strobe, LED bar, and so on.                                                                                    |
| **Fixture category**    | Classification in the fixture picker: Moving Head, Spot, Wash, Beam, PAR, Strobe, Batten, ...                                                                        |
| **Fixture profile**     | Definition of a fixture's channels, modes and characteristics, taken from a GDTF file.                                                                               |
| **Flash**               | Momentary button press that pushes a group: Simple, White, Strobe or Random.                                                                                         |
| **GDTF**                | General Device Type Format (DIN SPEC 15800): standard describing a fixture, its modes, channels and physical ranges.                                                 |
| **GDTF-Share**          | Online library of GDTF profiles used by the application to search and download fixtures.                                                                             |
| **Gobo**                | Mask or pattern projected by a projector.                                                                                                                            |
| **Grand Master**        | Global fader that scales the intensity of every group through the All group.                                                                                         |
| **Group**               | Set of fixtures driven together (master, colour, effects, flashes). A fixture can belong to several groups.                                                          |
| **Group count**         | Number of zones (K) used when the Grouped pixel preference is selected.                                                                                              |
| **Intensity**           | Light level of a fixture or group, driven by the master and the dimmer.                                                                                              |
| **Inverse Pan**         | Reverses the horizontal rotation direction of the group.                                                                                                             |
| **Inverse Tilt**        | Reverses the vertical rotation direction of the group.                                                                                                               |
| **KiNET**               | Philips Color Kinetics protocol over UDP (port 6038).                                                                                                                |
| **Master**              | Intensity fader of a group.                                                                                                                                          |
| **Max Power**           | Power limit of a group (0 to 100 %). It caps intensity, including during flashes and strobes.                                                                        |
| **Mode**                | Channel configuration of a fixture (number and role of its channels), defined in its GDTF profile.                                                                   |
| **Moving Head**         | Motorised projector able to pan and tilt.                                                                                                                            |
| **MVR**                 | My Virtual Rig (DIN SPEC 15801): scene exchange format describing layers, fixture instances and DMX addressing. It can be imported into the application.             |
| **OPC**                 | Open Pixel Control, a TCP protocol (port 7890).                                                                                                                      |
| **OSC**                 | Open Sound Control, a UDP protocol (port 8000) used by the application to send DMX.                                                                                  |
| **Output**              | Screen that routes universes to a network protocol or a physical interface.                                                                                          |
| **Output target**       | Destination of a universe: none, Art-Net, sACN, OSC, KiNET, OPC or a physical interface.                                                                             |
| **Pan**                 | Horizontal rotation of a moving head.                                                                                                                                |
| **Pan bounds**          | Allowed angular range for the group's pan (minimum and maximum).                                                                                                     |
| **Patch**               | Action and screen that associate fixtures with universes and DMX addresses, and assign them to groups.                                                               |
| **Physical interface**  | USB DMX box connecting the application to a physical DMX line, for example an Enttec DMX USB Pro.                                                                    |
| **Pixel**               | Individual light cell of a multi-cell fixture (LED bar, batten, strip).                                                                                              |
| **Pixel preference**    | How a group's pixels are driven: Single (one step per fixture, identical pixels), Individual (one step per pixel, global sequence) or Grouped (K zones per fixture). |
| **sACN (E1.31)**        | Streaming ACN (ANSI E1.31), a protocol sending DMX over UDP (port 5568).                                                                                             |
| **Show**                | Complete project of the application: patch, groups, configuration, outputs and fixture definitions.                                                                  |
| **Shutter**             | Shutter channel of a fixture (open or closed).                                                                                                                       |
| **Strobe**              | Rapid blinking of a fixture. Also names the flash type and the matching fixture category.                                                                            |
| **Tilt**                | Vertical rotation (inclination) of a moving head.                                                                                                                    |
| **Tilt bounds**         | Allowed angular range for the group's tilt (minimum and maximum).                                                                                                    |
| **Universe**            | Set of 512 DMX channels identified by a number. Each universe is routed to an output (network protocol or physical interface).                                       |
| **Visualizer**          | Graphical representation of a universe's patch, showing fixtures and their addresses.                                                                                |
| **Width**               | Share of an effect's cycle (0 to 100 %) during which the effect is active.                                                                                           |
| **.ldmx file**          | Live DMX show file (JSON) containing the patch, groups, configuration, outputs and fixture definitions.                                                              |
