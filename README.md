# stuart

stuart - **S**imple **T**erminal **UART** is a TUI for communicating with serial ports


## Features

- **Keep Open** - Ports are kept open when a device disconnects and automatically reconnect when the device is back
- **VT100/ANSI emulation** - full color and cursor support
- **Local Echo** - show typed text for devices that don't return it themselves
- **Scrollback buffer** - up to 10,000 lines, mouse and keyboard scrolling
- **Insert / Control modes** - vim-inspired controls
- **Line Mode** - send every character instantly or a whole line at once
- **Config file** - configure defaults for interacting with devices in a central config file
- **Port Selection** - Select available ports and view information about them
- **Pause connection** - Connection can be paused, freeing the port for flashing firmware or other operations
- **Clipboard copy** - copy entire scrollback with `c`
- **Flush scrollback** - clear the screen with `f`
- **Settings dialogue** - baud rate, data bits, stop bits, parity, flow control; changes apply immediately
- **Shell completions** - bash, zsh, fish, elvish, powershell

### Demo
![demo](docs/demo.gif)

## Install

### Arch Linux (AUR)

```sh
# Binary package
yay -S stuart-bin

# Build from source
yay -S stuart
```

### Cargo

```sh
# will install 'stuart' binary
cargo install stuart-cli
```

### Prebuilt binaries

Download from the [releases page](https://github.com/mroetsc/stuart/releases) for Linux, Windows (x86_64) and macOS (aarch64).

## Usage

```text
Usage: stuart [OPTIONS] [PORT]

Arguments:
  [PORT]  Serial port to open

Serial Settings:
  -b, --baud <BAUDRATE>      Baud rate
  -d, --data-bits <BITS>     Data bits [possible values: 5, 6, 7, 8]
  -s, --stop-bits <BITS>     Stop bits [possible values: 1, 2]
  -p, --parity <PARITY>      Parity [possible values: none, even, odd]
  -f, --flow-control <FLOW>  Flow control [possible values: none, software, hardware]

Behavior:
  -e, --local-echo
          Echo typed characters locally (for devices that don't echo)
      --input-mode <MODE>
          Send every character instantly or a whole line at once [possible values: direct, line]
      --outgoing-newline <NEWLINE_ENCODING>
          Encoding to send to the device when pressing Enter [possible values: cr, lf, crlf]
      --incoming-newline <NEWLINE_ENCODING>
          Newline encoding expected from the device [possible values: cr, lf, crlf]
      --no-lock
          Don't lock the port
  -k, --keep-open
          Keep terminal open and reconnect if the device disconnects [default]
      --no-keep-open
          Exit to port select when device disconnects

Extra:
      --create-config        Write a default config file
      --completions <SHELL>  Generate shell completions [possible values: bash, elvish, fish, powershell, zsh]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

**Examples:**

```sh
# open port selection screen
stuart

# connect directly
stuart /dev/ttyACM0

# connect at 9600 baud
stuart /dev/ttyUSB0 -b 9600
stuart -b 9600 /dev/ttyUSB0
```

## Key Bindings

### Port Selection

| Key | Action |
|-----|--------|
| `↑` / `↓` / `j` / `k` | Navigate |
| `Enter` | Open port |
| `s` | Settings |
| `r` | Refresh port list |
| `q` | Quit |

### Terminal - Control Mode

Enter control mode with `Ctrl+Esc` or `Ctrl+Space`.
> [!NOTE]
> For `Ctrl+Esc` to work, your terminal emulator has to support the [kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/), which most modern emulators do. If not, only `Ctrl+Space` will work.


| Key | Action |
|-----|--------|
| `a` / `i` | Enter insert mode |
| `↑` / `↓` / `j` / `k` | Scroll |
| `esc` | Scroll to bottom |
| `f` | Flush screen |
| `c` | Copy scrollback to clipboard |
| `+` / `-` | Cycle baud rate |
| `p` | Pause port |
| `s` | Settings |
| `del` | Disconnect → port select |
| `q` | Quit |

### Terminal - Insert Mode

All keypresses are forwarded to the device. Press `Ctrl+Esc` to return to control mode.

### Settings

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate |
| `←` / `→` | Cycle value |
| `Enter` | Edit baud rate (type custom value) |
| `Esc` | Close |

## Building from Source

```sh
git clone https://github.com/yourusername/stuart
cd stuart
cargo build --release
# binary at target/release/stuart
```

## Configuration
Use the `--create-config` argument to create a new default configuration. Priority order of connfiguration values is: Application Defaults -> Config file -> CLI arguments, with CLI arguments having the highest priority.

## Shell Completions

```sh
# zsh
stuart --completions zsh > ~/.zfunc/_stuart

# bash
stuart --completions bash > /etc/bash_completion.d/stuart

# fish
stuart --completions fish > ~/.config/fish/completions/stuart.fish

# elvish and powershell also supported
```

## Contributing
Contributions are welcome, but make sure to file an [issue](https://github.com/mroetsc/stuart/issues) first to discuss features and implementation.
