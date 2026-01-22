# cli-t

A minimal real-time CLI chat application built with Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- Room-based chat with unique IDs
- Public and private rooms with password protection
- Room capacity limits and persistent rooms
- Custom or random nicknames
- Real-time message delivery with timestamps
- Message history (last 50 messages on join)
- Admin and moderator system with kick/ban
- Color themes and message formatting
- Auto-reconnect on disconnect
- Sound notifications
- Command history
- Message logging
- Rate limiting and input sanitization
- Server metrics display
- Graceful shutdown
- Simple slash commands

## Installation

### Pre-built Binaries

**Quick Install (Linux/macOS):**

```bash
curl -fsSL https://raw.githubusercontent.com/Nuu-maan/cli-t/main/install.sh | bash
```

**Quick Install (Windows PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Nuu-maan/cli-t/main/install.ps1 | iex
```

**Manual Download:**

Download from [GitHub Releases](https://github.com/Nuu-maan/cli-t/releases):

| Platform | Architecture | Download |
|----------|--------------|----------|
| Windows | x64 | [cli-t-x86_64-pc-windows-msvc.zip](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-x86_64-pc-windows-msvc.zip) |
| Linux | x64 | [cli-t-x86_64-unknown-linux-gnu.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-x86_64-unknown-linux-gnu.tar.gz) |
| macOS | Intel | [cli-t-x86_64-apple-darwin.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-x86_64-apple-darwin.tar.gz) |
| macOS | Apple Silicon | [cli-t-aarch64-apple-darwin.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-aarch64-apple-darwin.tar.gz) |

Extract and optionally add to PATH:

```bash
tar -xzf cli-t-*.tar.gz
sudo mv cli-t /usr/local/bin/  # optional
```

### Build from Source

Requires Rust 1.70+

```bash
git clone https://github.com/Nuu-maan/cli-t.git
cd cli-t
cargo build --release
```

Binaries will be in `target/release/` (cli-t and cli-t-server).

## Configuration

Create `config.toml` in the same directory as the client:

```toml
[server]
ip = "127.0.0.1"
port = "8080"

[theme]
username_color = "cyan"
message_color = "white"
system_color = "yellow"
timestamp_color = "bright_black"

[features]
colors = true
timestamps = true
sound_notifications = true
auto_reconnect = true
save_logs = false
```

**Command-line override:**

You can also override the server address via command line:

```bash
cli-t --server <ip> [port]
```

## Quick Start

Start the server:

```bash
./cli-t-server              # default: 127.0.0.1:8080
./cli-t-server 0.0.0.0:8080  # custom address
```

In another terminal, start the client:

```bash
./cli-t
```

Create or join a room:

```
> /create
Room created: room-abc12 (public)
Share this ID with others to join.

> /join room-abc12
Joined room: room-abc12
```

## Commands

### Room Management

| Command | Description | Aliases |
|---------|-------------|---------|
| `/create [public\|private] [password <pwd>] [capacity <n>] [persistent]` | Create a new room | `/c` |
| `/join <id> [password]` | Join an existing room | `/j` |
| `/list` | List all public rooms | `/l` |
| `/quit` | Leave the current room | `/q` |

### User Management

| Command | Description |
|---------|-------------|
| `/users` | Show users in current room | `/u` |
| `/nick <name>` | Change your nickname |
| `/me <action>` | Send an action message (e.g., `/me waves`) |

### Moderation (Admin/Mod only)

| Command | Description |
|---------|-------------|
| `/kick <user>` | Kick a user from the room |
| `/ban <user>` | Ban a user from the room |

### Help

| Command | Description | Aliases |
|---------|-------------|---------|
| `/help` | Show available commands | `/h` |

## Advanced Features

### Room Types

**Public Rooms:**
- Visible in `/list` command
- Anyone can join (if no password)

**Private Rooms:**
- Not listed in `/list`
- Must know the exact room ID to join

**Password Protection:**
```bash
> /create private password secret123
Room created: room-xyz45 (private)
Password protected
```

**Capacity Limits:**
```bash
> /create capacity 10
Room created: room-abc12 (public)
Capacity: 10
```

**Persistent Rooms:**
```bash
> /create persistent
Room created: room-abc12 (public)
# Room persists even when empty
```

### Admin System

- Room creator automatically becomes admin
- Admins can kick and ban users
- Admins can kick other admins (but not themselves)
- Moderators can kick users (but not admins)

### Message History

When you join a room, you'll see the last 50 messages automatically.

### Color Themes

Customize colors in `config.toml`:

```toml
[theme]
username_color = "cyan"      # Options: black, red, green, yellow, blue, magenta, cyan, white, bright_*
message_color = "white"
system_color = "yellow"
timestamp_color = "bright_black"
```

### Auto-Reconnect

If enabled, the client will automatically attempt to reconnect if the connection is lost (up to 5 attempts).

### Message Logging

Enable message logging in `config.toml`:

```toml
[features]
save_logs = true
```

Logs are saved to `cli-t-YYYYMMDD.log` in the current directory.

### Sound Notifications

Sound notifications play when new messages arrive (can be disabled in config).

## Self-Hosting

Run your own server:

```bash
./cli-t-server              # default: 127.0.0.1:8080
./cli-t-server 0.0.0.0:8080  # listen on all interfaces
```

Configure clients to connect:

```toml
[server]
ip = "your-server-ip"
port = "8080"
```

The server displays real-time metrics:
- Total users
- Active users
- Total messages
- Number of rooms
- Uptime

## Security Features

- Rate limiting (10 messages per second per user)
- Input sanitization (removes control characters)
- Message length limits (1000 characters)
- Nickname length limits (32 characters)
- Password-protected rooms

## Contributing

Contributions welcome. Fork the repository, make your changes, and submit a pull request.

Development:

```bash
cargo run --package cli-t-server  # run server
cargo run --package cli-t         # run client
cargo test                        # run tests
cargo fmt                         # format code
cargo clippy                      # lint code
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with Rust and Tokio.
