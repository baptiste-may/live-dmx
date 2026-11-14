//! Network DMX protocol senders (Art-Net, sACN, OSC, KiNET, OPC).
//!
//! Each submodule provides a pure packet builder and a sender that the live
//! output runner calls once per universe and tick. See
//! `docs/contributing/dmx-output.md` for the routing and protocol details.

/// Art-Net (ArtDmx) sender.
pub mod artnet;
/// KiNET sender.
pub mod kinet;
/// Open Pixel Control sender.
pub mod opc;
/// OSC sender.
pub mod osc;
/// sACN (E1.31) sender.
pub mod sacn;

pub use artnet::*;
pub use kinet::*;
pub use opc::*;
pub use osc::*;
pub use sacn::*;
