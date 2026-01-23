# cli-t Updates - Comprehensive Changelog

## Overview
This document details all the updates and enhancements made to cli-t today. The application has been significantly enhanced with new features, improved user experience, and better integration capabilities.

---

## Major Feature Additions

### 1. Advanced Room Management System

#### Room Types
- **Public Rooms**: Visible in `/list` command, anyone can join (if no password)
- **Private Rooms**: Not listed publicly, requires exact room ID to join
- **Command**: `/create [public|private]`

#### Room Protection & Limits
- **Password Protection**: Secure rooms with passwords
  - Command: `/create [private] password <pwd>`
  - Join: `/join <room-id> <password>`
- **Capacity Limits**: Set maximum users per room
  - Command: `/create capacity <n>`
- **Persistent Rooms**: Rooms that survive when empty
  - Command: `/create persistent`
  - Useful for permanent chat spaces

#### Room Listing
- **New Command**: `/list` (alias: `/l`)
  - Lists all public rooms
  - Shows user count and capacity for each room
  - Helps users discover available chat spaces

### 2. Moderation System

#### Admin & Moderator Roles
- **Room Creator**: Automatically becomes admin
- **Admin Permissions**:
  - Kick users: `/kick <user>`
  - Ban users: `/ban <user>`
  - Can kick other admins (but not themselves)
- **Moderator Permissions**:
  - Kick users (but not admins)
  - Assigned by admins (future feature)

#### Moderation Commands
- `/kick <user>` - Remove user from room (mod/admin)
- `/ban <user>` - Permanently ban user from room (admin only)
- `/users` (alias: `/u`) - View all users in current room with roles

### 3. Message History & Timestamps

#### Message History Buffer
- **Last 50 Messages**: Automatically shown when joining a room
- **Storage**: Up to 1000 messages per room kept in memory
- **Format**: Includes timestamps, usernames, and message content
- **Action Messages**: `/me <action>` commands stored in history

#### Timestamps
- **Format**: `[HH:MM]` on all messages
- **Display**: Shows when each message was sent
- **Color**: Customizable via theme configuration

### 4. Enhanced User Experience

#### Color Themes
- **Customizable Colors**: Full theme support via `config.toml`
  - Username color
  - Message color
  - System message color
  - Timestamp color
- **Color Options**: black, red, green, yellow, blue, magenta, cyan, white, bright_*
- **Default Theme**: Cyan usernames, white messages, yellow system, bright_black timestamps

#### Command Aliases
- `/create` → `/c`
- `/join` → `/j`
- `/list` → `/l`
- `/quit` → `/q`
- `/users` → `/u`
- `/help` → `/h`

#### Action Messages
- **New Command**: `/me <action>`
  - Example: `/me waves` → `* username waves`
  - Formatted with asterisk prefix
  - Stored in message history

#### Nickname Management
- **New Command**: `/nick <name>`
  - Change nickname on the fly
  - Validated (max 32 characters)
  - Notifies room members of change

### 5. Security & Performance Enhancements

#### Rate Limiting
- **Limit**: 10 messages per second per user
- **Protection**: Prevents spam and abuse
- **Response**: Clear error message when limit exceeded

#### Input Sanitization
- **Control Characters**: Automatically removed
- **Message Length**: Limited to 1000 characters
- **Nickname Length**: Limited to 32 characters
- **Validation**: All user input sanitized before processing

### 6. Server Metrics & Monitoring

#### Real-time Metrics Display
- **Console Output**: Updates every 5 seconds
- **Metrics Shown**:
  - Total users (all time)
  - Active users (currently connected)
  - Total messages sent
  - Number of active rooms
  - Server uptime

#### Discord Webhook Integration
- **Automatic Updates**: Every 5 minutes
- **Single Message**: Updates same message (no spam)
- **Rich Embed**: Beautiful formatted metrics card
- **Metrics Displayed**:
  - Total and active users
  - Total messages
  - Active rooms
  - Uptime (formatted: hours, minutes, seconds)
  - Last update timestamp
- **Configuration**: Set via `DISCORD_WEBHOOK` environment variable
- **Default Webhook**: Pre-configured for production use

### 7. Client-Side Improvements

#### Windows Notifications
- **Toast Notifications**: Minimal popups in bottom-right corner
- **Smart Formatting**: Extracts username and message
- **Message Truncation**: Long messages (>50 chars) truncated
- **Non-Spammy**: Only shows for actual chat messages (not system messages)
- **Platform**: Windows 10/11 native toast notifications

#### Smart Sound Notifications
- **Background Mode**: Only plays sound when window is NOT active
- **Active Window Detection**: Checks if terminal is in foreground
- **Configurable**: Can be disabled via `config.toml`
- **Platform Support**: Windows beep, cross-platform fallback

#### Auto-Reconnect
- **Automatic**: Reconnects on connection loss
- **Retry Logic**: Up to 5 attempts with 3-second delays
- **Configurable**: Can be disabled via `config.toml`
- **User Feedback**: Clear messages about reconnection status

#### Command History
- **Storage**: Last 100 commands remembered
- **Navigation**: Reserved for future keyboard input (arrow keys)
- **Deduplication**: Prevents duplicate consecutive commands

#### Message Logging
- **Optional**: Can be enabled via `config.toml`
- **File Format**: `cli-t-YYYYMMDD.log`
- **Location**: Current directory
- **Content**: All messages with timestamps

### 8. Configuration Improvements

#### Enhanced Config File Search
- **Multiple Locations**: Searches in order:
  1. Current working directory
  2. Parent directories (up to 5 levels) - finds project root
  3. Executable directory
  4. Parent directories from executable (up to 5 levels)
  5. User home directory
- **Developer Friendly**: Works when running from subdirectories
- **Production Ready**: Finds config when binary is in any location

#### Command-Line Override
- **Usage**: `cli-t --server <ip> [port]`
- **Override**: Bypasses `config.toml` if needed
- **Quick Testing**: Easy way to test different servers

#### Configuration Options
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

---

## Technical Improvements

### Code Quality
- **Dead Code Warnings**: Fixed all compiler warnings
- **Reserved Fields**: Marked with `#[allow(dead_code)]` for future features
- **Error Handling**: Comprehensive error handling throughout
- **Code Formatting**: All code formatted with `cargo fmt`

### Dependencies Added
- **Server**:
  - `reqwest` - HTTP client for Discord webhooks
  - `chrono` - Timestamp handling
- **Client**:
  - `colored` - Terminal color support
  - `regex` - Message parsing and formatting
  - `chrono` - Timestamp handling
  - `dirs` - Home directory detection
  - `winrt-notification` - Windows toast notifications
  - `winapi` - Windows API for window detection

### Architecture
- **Async/Await**: Full async implementation with Tokio
- **Thread Safety**: Proper use of `Arc<RwLock<>>` for shared state
- **Message Broadcasting**: Efficient broadcast channels for room messages
- **Task Management**: Proper task spawning and cleanup

---

## Bug Fixes

### Server
- Fixed message echoing (messages no longer echo back to sender)
- Fixed room cleanup (empty rooms properly removed)
- Fixed nickname handling (proper synchronization across tasks)
- Fixed rate limiting (proper async implementation)

### Client
- Fixed config file location (finds config in multiple locations)
- Fixed connection handling (better error messages)
- Fixed message display (proper formatting with colors)
- Fixed window detection (proper foreground window checking)

---

## Commands Reference

### Room Management
| Command | Description | Aliases |
|---------|-------------|---------|
| `/create [public\|private] [password <pwd>] [capacity <n>] [persistent]` | Create room | `/c` |
| `/join <id> [password]` | Join room | `/j` |
| `/list` | List public rooms | `/l` |
| `/quit` | Leave room | `/q` |

### User Management
| Command | Description |
|---------|-------------|
| `/users` | Show room users | `/u` |
| `/nick <name>` | Change nickname |
| `/me <action>` | Action message |

### Moderation
| Command | Description | Permission |
|---------|-------------|-----------|
| `/kick <user>` | Kick user | Mod/Admin |
| `/ban <user>` | Ban user | Admin only |

### Help
| Command | Description | Aliases |
|---------|-------------|---------|
| `/help` | Show commands | `/h` |

---

## Release Information

### Version 0.2.2
- **Tag**: `v0.2.2`
- **Date**: Today
- **Highlights**:
  - Discord webhook metrics integration
  - Windows toast notifications
  - Production server configuration in binaries

### Version 0.2.1
- **Tag**: `v0.2.1`
- **Highlights**:
  - Production server configuration

### Version 0.2.0
- **Tag**: `v0.2.0`
- **Highlights**:
  - Comprehensive room management
  - Moderation system
  - Message history
  - Enhanced UI features

---

## Migration Guide

### For Users
1. **Update Config**: Add new `[theme]` and `[features]` sections to `config.toml`
2. **New Commands**: Learn new commands like `/list`, `/users`, `/me`, `/nick`
3. **Room Features**: Try creating password-protected or capacity-limited rooms

### For Developers
1. **Dependencies**: Run `cargo build` to fetch new dependencies
2. **Environment**: Set `DISCORD_WEBHOOK` for server metrics (optional)
3. **Build**: All platforms supported (Windows, Linux, macOS)

---

## Future Enhancements (Reserved)

The following features are reserved for future implementation:
- Typing indicators
- Emoji support
- Command history navigation (arrow keys)
- Timestamp toggle feature
- Room age/analytics
- Advanced logging and moderation features

---

## Performance Metrics

### Server
- **Concurrent Users**: Tested with multiple simultaneous connections
- **Message Throughput**: Handles high message rates with rate limiting
- **Memory Usage**: Efficient message history storage (1000 messages max per room)
- **CPU Usage**: Minimal overhead with async architecture

### Client
- **Connection Speed**: Fast connection establishment
- **Reconnection**: Automatic with configurable retries
- **Notification Latency**: Near-instant Windows notifications
- **Resource Usage**: Lightweight client implementation

---

## Documentation Updates

### README.md
- Updated with all new features
- Added command reference table
- Added configuration examples
- Added advanced features documentation
- Added security features section

### Config Examples
- Default local development config
- Production server config (in binaries)
- Theme customization examples
- Feature toggle examples

---

## Testing

### Tested Scenarios
- ✅ Multiple users in same room
- ✅ Room creation with various options
- ✅ Password-protected rooms
- ✅ Capacity limits
- ✅ Moderation commands
- ✅ Message history on join
- ✅ Auto-reconnect on disconnect
- ✅ Windows notifications
- ✅ Discord webhook updates
- ✅ Config file location detection
- ✅ Command-line server override

### Known Limitations
- Typing indicators not yet implemented
- Emoji support not yet implemented
- Command history navigation (arrow keys) not yet implemented
- Cross-platform window detection limited to Windows

---

## Contributors

All updates implemented today with comprehensive testing and documentation.

---

## License

MIT License - See LICENSE file for details.

---

*Last Updated: Today*
*Version: 0.2.2*
