# cli-t

A minimal real-time CLI chat application built with Rust. Create rooms, invite friends, and chat from your terminal.

## Features

- **Room-based chat** - Create and join chat rooms with unique IDs
- **Custom nicknames** - Choose your own or get a random one
- **Real-time messaging** - Instant message delivery
- **Simple commands** - Easy-to-use CLI interface
- **Lightweight** - Fast and minimal dependencies

## Quick Install

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Nuu-maan/cli-t/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Nuu-maan/cli-t/main/install.ps1 | iex
```

The install script will build cli-t from source (requires Rust). After installation, restart your terminal and you're ready to go!

## Manual Installation

### From Source

```bash
git clone https://github.com/Nuu-maan/cli-t.git
cd cli-t

# Build and install
cargo install --path server
cargo install --path client
```

### Pre-built Binaries

Download binaries from [GitHub Releases](https://github.com/Nuu-maan/cli-t/releases) and add them to your PATH.

## Quick Start

1. **Start the server:**
   ```bash
   cli-t-server
   ```
   Server listens on `127.0.0.1:8080` by default.

2. **Start a client:**
   ```bash
   cli-t
   ```

3. **Create or join a room:**
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

## Custom Server Address

Both server and client accept a command-line argument for the address:

```bash
# Server: bind to specific address
cli-t-server 0.0.0.0:8080

# Client: connect to specific server
cli-t example.com:8080
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
cd server && cargo run

# Run client (in another terminal)
cd client && cargo run
```

Feel free to open an issue if you have questions or suggestions!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with Rust and Tokio.

---

**Made with care for the terminal community**
