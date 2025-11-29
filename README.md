# cli-t

A minimal real-time CLI chat application in Rust with room-based messaging, nicknames, and commands.

## Installation

### From crates.io (Recommended)

If published to crates.io, users can install with:

```bash
# Install server
cargo install cli-t-server

# Install client
cargo install cli-t
```

The binaries will be available in `~/.cargo/bin/` (or `%USERPROFILE%\.cargo\bin` on Windows).

### From Source

```bash
# Clone the repository
git clone https://github.com/Nuu-maan/cli-t.git
cd cli-t

# Install both components
cargo install --path server
cargo install --path client
```

### Pre-built Binaries (Alternative)

Download pre-built binaries from [GitHub Releases](https://github.com/Nuu-maan/cli-t/releases) and add them to your PATH.

## Quick Start

1. **Start the server** (in one terminal):
   ```bash
   cli-t-server
   ```
   The server listens on `127.0.0.1:8080` by default.

2. **Start clients** (in additional terminals):
   ```bash
   cli-t
   ```

3. **Follow the prompts**:
   - Enter a nickname (or leave blank for random)
   - Use `/create` to create a room
   - Use `/join <room-id>` to join an existing room
   - Type messages to chat
   - Use `/quit` to leave a room
   - Use `/help` to see all commands

## Usage Example

```
$ cli-t

Welcome to cli-t!

Nick (leave blank for random): coolguy

You are: coolguy

Commands:
  /create       - Create a new room
  /join <id>    - Join existing room
  /quit         - Leave room
  /help         - Show commands

> /create
Room created: room-x7k9p
Share this ID with others to join.

[coolguy joined]

> hey what's up

> /quit
Left the room.

>
```

## Commands

- `/create` - Create a new room and get a room ID
- `/join <id>` - Join an existing room by ID
- `/quit` - Leave the current room
- `/help` - Show available commands

## Features

- Room-based chat system
- Custom nicknames (or random if left blank)
- Real-time message broadcasting within rooms
- Multiple concurrent clients
- Simple text-based protocol
- Async I/O with Tokio
- Graceful disconnection handling

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/Nuu-maan/cli-t.git
cd cli-t

# Build both projects
cargo build --release

# Run server
cd server && cargo run

# Run client (in another terminal)
cd client && cargo run [server-address]
```

### Custom Server Address

Both server and client accept a command-line argument for the address:

```bash
# Server: bind to specific address
cargo run -- 0.0.0.0:8080

# Client: connect to specific server
cargo run -- example.com:8080
```

## Project Structure

```
cli-t/
├── Cargo.toml          # Workspace configuration
├── server/             # Server project
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── client/             # Client project
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Publishing to crates.io

To publish to crates.io:

1. Create accounts on [crates.io](https://crates.io) and get an API token
2. Repository URLs are already set in `server/Cargo.toml` and `client/Cargo.toml`
3. Publish:
   ```bash
   cd server && cargo publish
   cd ../client && cargo publish
   ```

## License

MIT
