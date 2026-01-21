# prism-tui

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)

A terminal user interface for [PrismLauncher](https://prismlauncher.org/) - manage and launch your Minecraft instances from the command line.

## Features

- **Instance Management** - Browse all your PrismLauncher instances with version and mod loader info
- **Quick Launch** - Launch instances directly from the terminal
- **Server Management** - View, add, edit, and delete servers for any instance
- **Join on Launch** - Configure instances to auto-join a server when launched
- **Account Selection** - Switch between accounts before launching
- **Log Viewer** - Browse and read instance and launcher logs
- **Search** - Filter instances and accounts with incremental search
- **Vim-style Navigation** - `j/k` navigation with optional arrow key support

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/OneNoted/prism-tui.git
cd prism-tui

# Build and install
cargo install --path .
```

### Requirements

- Rust 1.92.0 or newer
- PrismLauncher installed with at least one instance

## Usage

```bash
prism-tui
```

The application automatically detects your PrismLauncher data directory.

### Keybindings

#### Global
| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Show help |
| `Ctrl+T` | Toggle vim mode |

#### Instances Screen
| Key | Action |
|-----|--------|
| `j` / `k` or `Up` / `Down` | Navigate instances |
| `l` / `Enter` | Launch selected instance |
| `g` / `G` | Go to first / last instance |
| `gl` | Open launcher logs |
| `L` | Open instance logs |
| `s` | Open server list |
| `a` | Select account |
| `o` | Open instance folder |
| `/` | Search instances |

#### Servers Screen
| Key | Action |
|-----|--------|
| `j` / `k` | Navigate servers |
| `l` / `Enter` | Launch with selected server |
| `a` | Add new server |
| `e` | Edit server |
| `d` | Delete server |
| `J` | Toggle join-on-launch |
| `h` / `Esc` | Back |

#### Accounts Screen
| Key | Action |
|-----|--------|
| `j` / `k` | Navigate accounts |
| `l` / `Enter` | Select account |
| `h` / `Esc` | Back |
| `/` | Search accounts |

#### Logs Screen
| Key | Action |
|-----|--------|
| `j` / `k` | Navigate log files |
| `l` / `Enter` | View log content |
| `J` / `K` or `PageDown` / `PageUp` | Scroll content |
| `e` | Open in editor |
| `o` | Open logs folder |
| `h` / `Esc` | Back |

### Search

Press `/` to enter search mode. Type to filter the list incrementally. Press `Enter` to confirm or `Esc` to cancel.

## Configuration

prism-tui reads its configuration from PrismLauncher's data directory:

- **Linux**: `~/.local/share/PrismLauncher/`
- **macOS**: `~/Library/Application Support/PrismLauncher/`
- **Windows**: `%APPDATA%/PrismLauncher/`

No additional configuration is required.

## Architecture

prism-tui uses an Elm-style architecture with:

- **App state** (`src/app.rs`) - Central state management
- **Messages** (`src/message.rs`) - All possible actions
- **Update** (`src/update.rs`) - Message handlers and keybindings
- **Views** (`src/view/`) - UI rendering per screen

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [PrismLauncher](https://prismlauncher.org/) - The launcher this TUI interfaces with
- [Ratatui](https://ratatui.rs/) - Terminal UI framework
