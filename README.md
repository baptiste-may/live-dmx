# Live DMX 🎭⚡

**Live DMX** is a lightning-fast lighting control application designed for live performance environments. It bridges the gap between complex lighting consoles and the need for immediate, intuitive control, allowing lighting designers to go from "zero to show" in seconds.

## 🚀 The Mission

Modern lighting consoles often require hours of patching and programming before a single light can be moved. **Live DMX** focuses on reducing this overhead by providing a streamlined workflow for:

- **Instant Patching:** Quickly add and address your fixtures.
- **Rapid Programming:** Minimal setup for live buttons, faders, and playbacks.
- **Live Performance Focus:** Built for the heat of the moment, with tactile-friendly controls.

## ✨ Key Features

- **📡 Real-time DMX Output:** Engine computes and generates active DMX universe frames at 44Hz.
- **🛠️ Integrated Fixture Library:** Access a growing library of DMX fixture definitions (JSON-based) or create your own.
- **🖥️ Dedicated Workspaces:**
  - **Input:** Handle external control sources.
  - **Output:** Monitor your DMX universes in real-time.
  - **Patch:** Manage your fixture inventory and DMX addresses.
  - **Configure:** Setup your show's logic and mappings.
  - **Live:** The performance dashboard with buttons, faders, and playbacks.
- **💾 Show Management:** Save and load shows using the dedicated `.ldmx` file format.
- **🎨 Modern UI:** A clean, responsive interface built with Vue 3, Tailwind CSS, and DaisyUI.

## 🛠️ Tech Stack

- **Backend:** [Tauri v2](https://v2.tauri.app/) (Rust) for performance and system access.
- **Frontend:** [Vue 3](https://vuejs.org/) with TypeScript and Vite.
- **Styling:** [Tailwind CSS](https://tailwindcss.com/) & [DaisyUI](https://daisyui.com/).
- **Icons:** [Lucide Vue Next](https://lucide.dev/).

## 🚦 Getting Started

### Prerequisites

- [Bun](https://bun.sh/) (for package management and task running)
- [Rust](https://www.rust-lang.org/) (installed via rustup)
- Tauri dependencies (see the [Tauri setup guide](https://v2.tauri.app/start/prerequisites/))

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/baptiste-may/live-dmx.git
   cd live-dmx
   ```
2. Install dependencies:
   ```bash
   bun install
   ```

### Development

Run the application in development mode:

```bash
bun run dev:app
```

### Build

Generate a production-ready application:

```bash
bun run tauri build
```

## 🤝 Contributing

Contributions are welcome! If you're interested in adding new fixture definitions or improving the live interface, please check out our `AGENTS.md` for project-specific instructions and coding standards.

## 📄 License

This project is licensed under the **GNU General Public License v3.0 (GPLv3)** - see the [LICENSE](LICENSE) file for details.

© 2026 Baptiste May
