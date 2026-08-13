# SevenSSH

> A modern, secure SSH client built with Tauri — fast, lightweight, and privacy-first.

![SevenSSH Screenshot](docs/screenshot-placeholder.png)

## Features

- **SSH Terminal** — Full-featured terminal with xterm.js, split panes, and tabs
- **SFTP File Manager** — Drag-and-drop file transfers with progress tracking
- **AI Assistant** — Context-aware command suggestions (local Ollama or OpenAI)
- **Security First** — Master password encryption, biometric unlock, session locking
- **SSH Key Management** — Generate, import, and manage SSH keys
- **Connection Import** — Import from `~/.ssh/config` with one click
- **Cross-Platform** — macOS, Windows, and Linux support
- **Auto-Update** — Built-in update mechanism for seamless upgrades
- **Keyboard-Driven** — Extensive shortcuts for power users

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 9+
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- Platform-specific dependencies (see [Development](#development))

### Install & Run

```bash
# Clone the repository
git clone https://github.com/OWNER/sevenssh.git
cd sevenssh

# Install dependencies
pnpm install

# Start in development mode
pnpm tauri dev
```

### Build for Production

```bash
pnpm tauri build
```

Build artifacts will be in `src-tauri/target/release/bundle/`.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri 2](https://v2.tauri.app/) |
| Frontend | [Vue 3](https://vuejs.org/) + TypeScript |
| UI Library | [TDesign](https://tdesign.tencent.com/vue-next/) |
| Terminal | [xterm.js](https://xtermjs.org/) 5 |
| Backend | Rust (async SSH via `russh`) |
| State | [Pinia](https://pinia.vuejs.org/) |
| Build | [Vite](https://vitejs.dev/) 6 |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + T` | New tab |
| `Ctrl/Cmd + W` | Close tab |
| `Ctrl/Cmd + Tab` | Next tab |
| `Ctrl/Cmd + Shift + Tab` | Previous tab |
| `Ctrl/Cmd + D` | Split pane vertically |
| `Ctrl/Cmd + Shift + D` | Split pane horizontally |
| `Ctrl/Cmd + K` | Command palette |
| `Ctrl/Cmd + L` | Lock session |
| `Ctrl/Cmd + ,` | Settings |
| `Ctrl/Cmd + Shift + F` | SFTP panel |

## Development

### System Dependencies

**macOS:**
```bash
# Xcode Command Line Tools (includes required frameworks)
xcode-select --install
```

**Ubuntu/Debian:**
```bash
sudo apt install \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  libsoup-3.0-dev
```

**Windows:**
- [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11)

### Project Structure

```
sevenssh/
├── src/                    # Vue frontend
│   ├── components/         # UI components
│   ├── views/              # Page views
│   ├── stores/             # Pinia stores
│   ├── services/           # Tauri command wrappers
│   └── styles/             # Global styles & themes
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri command handlers
│   │   ├── ssh/            # SSH client implementation
│   │   ├── sftp/           # SFTP session management
│   │   ├── ai/             # AI integration (Ollama/OpenAI)
│   │   ├── crypto/         # Encryption utilities
│   │   └── db/             # SQLite database layer
│   └── icons/              # App icons
└── scripts/                # Build & utility scripts
```

### Available Scripts

```bash
pnpm dev          # Start Vite dev server
pnpm build        # Build frontend
pnpm tauri dev    # Start Tauri in dev mode (frontend + backend)
pnpm tauri build  # Production build
pnpm lint         # Lint frontend code
pnpm format       # Format frontend code
```

### Icon Generation

```bash
# Generate all icon sizes from a 1024x1024 source PNG
./scripts/generate-icons.sh path/to/icon-1024.png
```

## License

[MIT](LICENSE)
