# cli-t

A minimal real-time CLI chat application built with Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- Room-based chat with unique IDs
- Custom or random nicknames
- Real-time message delivery
- Simple slash commands
- Lightweight and fast
- Self-hostable

## Installation

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/Nuu-maan/cli-t/releases):

| Platform | Architecture | Download |
|----------|--------------|----------|
| Windows | x64 | [cli-t-x86_64-pc-windows-msvc.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-x86_64-pc-windows-msvc.tar.gz) |
| Linux | x64 | [cli-t-x86_64-unknown-linux-gnu.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-x86_64-unknown-linux-gnu.tar.gz) |
| Linux | ARM64 | [cli-t-aarch64-unknown-linux-gnu.tar.gz](https://github.com/Nuu-maan/cli-t/releases/latest/download/cli-t-aarch64-unknown-linux-gnu.tar.gz) |
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
```

## Quick Start

Start the server:

```bash
./cli-t-server
```

In another terminal, start the client:

```bash
./cli-t
```

Create or join a room:

```
> /create
Room created: room-abc12

> /join room-abc12
Joined room: room-abc12
```

## Commands

| Command | Description |
|---------|-------------|
| `/create` | Create a new room |
| `/join <id>` | Join an existing room |
| `/quit` | Leave the current room |
| `/help` | Show available commands |

## Self-Hosting

Run your own server:

```bash
./cli-t-server              # default: 127.0.0.1:8080
./cli-t-server 0.0.0.0:8080  # custom address
```

Configure clients to connect:

```toml
[server]
ip = "your-server-ip"
port = "8080"
```

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