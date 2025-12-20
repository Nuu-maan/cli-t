# cli-t

A minimal real-time CLI chat application built with Rust. Create rooms, invite friends, and chat from your terminal.

## Features

- **Room-based chat** - Create and join chat rooms with unique IDs
- **Custom nicknames** - Choose your own or get a random one
- **Real-time messaging** - Instant message delivery
- **Simple commands** - Easy-to-use CLI interface
- **Lightweight** - Fast and minimal dependencies

## Installation

### Using Pre-built Binaries (Recommended)

Download binaries for your platform from [GitHub Releases](https://github.com/Nuu-maan/cli-t/releases):

| Platform | Download |
|----------|----------|
| Windows (x64) | [cli-t-x86_64-pc-windows-msvc.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-x86_64-pc-windows-msvc.tar.gz) |
| Linux (x64) | [cli-t-x86_64-unknown-linux-gnu.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-x86_64-unknown-linux-gnu.tar.gz) |
| Linux (ARM64) | [cli-t-aarch64-unknown-linux-gnu.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-aarch64-unknown-linux-gnu.tar.gz) |
| macOS (Intel) | [cli-t-x86_64-apple-darwin.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-x86_64-apple-darwin.tar.gz) |
| macOS (Apple Silicon) | [cli-t-aarch64-apple-darwin.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-aarch64-apple-darwin.tar.gz) |

After downloading, extract the archive and configure your server address in `config.toml` (see Configuration section below).

### Self-Hosting from Source

```bash
git clone https://github.com/Nuu-maan/cli-t.git
cd cli-t
cargo build --release

# Binaries will be in target/release/
# - cli-t (client)
# - cli-t-server (server)
```

## Configuration

Create a `config.toml` file in the same directory as the client binary:

```toml
[server]
ip = "your-server-ip"
port = "8080"
```

Example for local testing:
```toml
[server]
ip = "127.0.0.1"
port = "8080"
```

## Quick Start

### Using Pre-built Binaries

1. **Download and extract** the binary for your platform
2. **Create `config.toml`** with your server address (see Configuration section)
3. **Run the client:**
    ```bash
    ./cli-t
    ```

### Self-Hosting

1. **Start the server:**
    ```bash
    cargo run --package cli-t-server
    # Or if built: ./target/release/cli-t-server
    ```
    Server listens on `127.0.0.1:8080` by default.

2. **Configure client** by creating `config.toml` with server address

3. **Start the client:**
    ```bash
    cargo run --package cli-t
    # Or if built: ./target/release/cli-t
    ```

4. **Create or join a room:**
   ```
   > /create
   Room created: room-abc12
   Share this ID with others to join.
   
   > /join room-abc12
   Joined room: room-abc12
   ```

4. **Start chatting!**

## Commands

| Command | Description |
|---------|-------------|
| `/create` | Create a new room and get a room ID |
| `/join <id>` | Join an existing room by ID |
| `/quit` | Leave the current room |
| `/help` | Show available commands |

## Usage Example

```
$ cli-t

Welcome to cli-t!

Nick (leave blank for random): alice

You are: alice

Commands:
  /create       - Create a new room
  /join <id>    - Join existing room
  /quit         - Leave room
  /help         - Show commands

> /create
Room created: room-xyz42
Share this ID with others to join.

[bob joined]
bob: Hey everyone!
alice: Hi bob!

> /quit
Left the room.
```

## Running Your Own Server

To host your own server:

```bash
# Using pre-built binary
./cli-t-server

# Or from source
cargo run --package cli-t-server

# Or specify bind address
./cli-t-server 0.0.0.0:8080
```

Then configure clients to connect by editing their `config.toml`:
```toml
[server]
ip = "your-server-ip"
port = "8080"
```

## Contributing

Contributions are welcome! Whether it's:

- Bug fixes
- New features
- Documentation improvements
- UI/UX enhancements
- Performance optimizations

### How to Contribute

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
git clone https://github.com/Nuu-maan/cli-t.git
cd cli-t

# Run server
cargo run --package cli-t-server

# Run client (in another terminal)
cargo run --package cli-t
```

Feel free to open an issue if you have questions or suggestions!

## Acknowledgments

Built with Rust and Tokio.

---

**Made with care for the terminal community**
MIT