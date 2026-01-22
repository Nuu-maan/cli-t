use colored::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

#[derive(Deserialize, Clone)]
struct Config {
    server: ServerConfig,
    #[serde(default)]
    theme: ThemeConfig,
    #[serde(default)]
    features: FeatureConfig,
}

#[derive(Deserialize, Default, Clone)]
struct ThemeConfig {
    #[serde(default = "default_username_color")]
    username_color: String,
    #[serde(default = "default_message_color")]
    message_color: String,
    #[serde(default = "default_system_color")]
    system_color: String,
    #[serde(default = "default_timestamp_color")]
    timestamp_color: String,
}

#[derive(Deserialize, Default, Clone)]
struct FeatureConfig {
    #[serde(default = "default_true")]
    colors: bool,
    #[serde(default = "default_true")]
    #[allow(dead_code)]
    timestamps: bool, // Reserved for future timestamp toggle feature
    #[serde(default = "default_true")]
    sound_notifications: bool,
    #[serde(default = "default_true")]
    auto_reconnect: bool,
    #[serde(default = "default_false")]
    save_logs: bool,
}

fn default_username_color() -> String {
    "cyan".to_string()
}

fn default_message_color() -> String {
    "white".to_string()
}

fn default_system_color() -> String {
    "yellow".to_string()
}

fn default_timestamp_color() -> String {
    "bright_black".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

#[derive(Deserialize, Clone)]
struct ServerConfig {
    ip: String,
    port: String,
}

struct ClientState {
    connected: bool,
    #[allow(dead_code)]
    current_room: Option<String>, // Reserved for future room status display
    command_history: VecDeque<String>,
    history_index: usize,
}

impl ClientState {
    fn new() -> Self {
        Self {
            connected: false,
            current_room: None,
            command_history: VecDeque::new(),
            history_index: 0,
        }
    }

    fn add_to_history(&mut self, cmd: String) {
        if !self.command_history.is_empty() && self.command_history.back() == Some(&cmd) {
            return;
        }
        self.command_history.push_back(cmd);
        if self.command_history.len() > 100 {
            self.command_history.pop_front();
        }
        self.history_index = self.command_history.len();
    }

    #[allow(dead_code)]
    fn history_up(&mut self) -> Option<String> {
        // Reserved for future keyboard input handling (arrow keys)
        if self.history_index > 0 {
            self.history_index -= 1;
            self.command_history.get(self.history_index).cloned()
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn history_down(&mut self) -> Option<String> {
        // Reserved for future keyboard input handling (arrow keys)
        if self.history_index < self.command_history.len() {
            self.history_index += 1;
            if self.history_index < self.command_history.len() {
                self.command_history.get(self.history_index).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn get_color_from_str(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "bright_black" => Color::BrightBlack,
        "bright_red" => Color::BrightRed,
        "bright_green" => Color::BrightGreen,
        "bright_yellow" => Color::BrightYellow,
        "bright_blue" => Color::BrightBlue,
        "bright_magenta" => Color::BrightMagenta,
        "bright_cyan" => Color::BrightCyan,
        "bright_white" => Color::BrightWhite,
        _ => Color::White,
    }
}

fn format_message(line: &str, config: &Config) -> String {
    if !config.features.colors {
        return line.to_string();
    }

    // Parse timestamp [HH:MM]
    let timestamp_re = Regex::new(r"^\[(\d{2}:\d{2})\]\s*").unwrap();
    let username_re = Regex::new(r"^\[(\d{2}:\d{2})\]\s*(\w+):\s*(.*)$").unwrap();
    let action_re = Regex::new(r"^\[(\d{2}:\d{2})\]\s*\*\s*(\w+)\s+(.*)$").unwrap();
    let system_re = Regex::new(r"^\[(\d{2}:\d{2})\]\s*\[(.*)\]$").unwrap();

    if let Some(caps) = action_re.captures(line) {
        let timestamp = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let username = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let action = caps.get(3).map(|m| m.as_str()).unwrap_or("");

        let ts_color = get_color_from_str(&config.theme.timestamp_color);
        let user_color = get_color_from_str(&config.theme.username_color);
        let msg_color = get_color_from_str(&config.theme.message_color);

        format!(
            "{} * {} {}\n",
            format!("[{}]", timestamp).color(ts_color),
            username.color(user_color),
            action.color(msg_color)
        )
    } else if let Some(caps) = username_re.captures(line) {
        let timestamp = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let username = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let message = caps.get(3).map(|m| m.as_str()).unwrap_or("");

        let ts_color = get_color_from_str(&config.theme.timestamp_color);
        let user_color = get_color_from_str(&config.theme.username_color);
        let msg_color = get_color_from_str(&config.theme.message_color);

        format!(
            "{} {}: {}\n",
            format!("[{}]", timestamp).color(ts_color),
            username.color(user_color),
            message.color(msg_color)
        )
    } else if let Some(caps) = system_re.captures(line) {
        let timestamp = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let system_msg = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        let ts_color = get_color_from_str(&config.theme.timestamp_color);
        let sys_color = get_color_from_str(&config.theme.system_color);

        format!(
            "{} {}\n",
            format!("[{}]", timestamp).color(ts_color),
            format!("[{}]", system_msg).color(sys_color)
        )
    } else if timestamp_re.is_match(line) {
        let ts_color = get_color_from_str(&config.theme.timestamp_color);
        timestamp_re
            .replace(line, |caps: &regex::Captures| {
                format!("{} ", format!("[{}]", &caps[1]).color(ts_color))
            })
            .to_string()
    } else {
        line.to_string()
    }
}

fn play_notification_sound() {
    #[cfg(windows)]
    {
        use std::process::Command;
        let _ = Command::new("powershell")
            .args(&["-Command", "[console]::beep(800,200)"])
            .output();
    }
    #[cfg(not(windows))]
    {
        use std::process::Command;
        let _ = Command::new("echo").arg("\x07").output();
    }
}

async fn connect_with_retry(
    addr: &str,
    max_retries: u32,
) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        match TcpStream::connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(
                        format!("Failed to connect after {} attempts: {}", max_retries, e).into(),
                    );
                }
                eprintln!(
                    "Connection failed (attempt {}/{}): {}. Retrying in 3 seconds...",
                    retries, max_retries, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments for server address override
    let args: Vec<String> = std::env::args().collect();
    let (config, addr) = if args.len() >= 3 && args[1] == "--server" {
        let addr = format!("{}:{}", args[2], args.get(3).unwrap_or(&"8080".to_string()));
        let config = Config {
            server: ServerConfig {
                ip: args[2].clone(),
                port: args.get(3).unwrap_or(&"8080".to_string()).clone(),
            },
            theme: ThemeConfig::default(),
            features: FeatureConfig::default(),
        };
        (config, addr)
    } else {
        // Read config from config.toml
        let config_content = match std::fs::read_to_string("config.toml") {
            Ok(content) => content,
            Err(_) => {
                eprintln!("Error: config.toml not found!");
                eprintln!("Usage: cli-t [--server <ip> [port]]");
                eprintln!("Or create a config.toml file with:");
                eprintln!("[server]");
                eprintln!("ip = \"your-server-ip\"");
                eprintln!("port = \"your-server-port\"");
                return Err("Config file not found".into());
            }
        };

        let config: Config = match toml::from_str(&config_content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error parsing config.toml: {}", e);
                return Err(e.into());
            }
        };

        let addr = format!("{}:{}", config.server.ip, config.server.port);
        (config, addr)
    };

    // Welcome message
    println!("{}", "Welcome to cli-t!".bright_cyan().bold());
    println!();

    // Get nickname
    print!("{}", "Nick (leave blank for random): ".bright_white());
    io::stdout().flush()?;

    let mut nickname = String::new();
    io::stdin().read_line(&mut nickname)?;
    let nickname = nickname.trim();

    let nickname = if nickname.is_empty() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("user-{:x}", timestamp % 10000)
    } else {
        nickname.to_string()
    };

    println!("{} {}\n", "You are:".bright_white(), nickname.bright_cyan());
    println!("{}", "Commands:".bright_white());
    println!("  {} - Create a new room", "/create".bright_green());
    println!("  {} - Join existing room", "/join <id>".bright_green());
    println!("  {} - List public rooms", "/list".bright_green());
    println!("  {} - Show room users", "/users".bright_green());
    println!("  {} - Action message", "/me <action>".bright_green());
    println!("  {} - Change nickname", "/nick <name>".bright_green());
    println!("  {} - Leave room", "/quit".bright_green());
    println!("  {} - Show commands\n", "/help".bright_green());

    // Setup log file if enabled
    let mut log_file = if config.features.save_logs {
        let log_path = PathBuf::from(format!(
            "cli-t-{}.log",
            chrono::Local::now().format("%Y%m%d")
        ));
        Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)?,
        )
    } else {
        None
    };

    let mut state = ClientState::new();
    let mut reconnect_attempts = 0;
    const MAX_RECONNECT_ATTEMPTS: u32 = 5;

    loop {
        println!("{}", "Connecting...".bright_yellow());

        // Connect to server
        let stream = match connect_with_retry(
            &addr,
            if config.features.auto_reconnect {
                MAX_RECONNECT_ATTEMPTS
            } else {
                1
            },
        )
        .await
        {
            Ok(stream) => {
                reconnect_attempts = 0;
                stream
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("Failed to connect to server at {}: {}", addr, e).bright_red()
                );
                eprintln!("{}", "Make sure the server is running.".bright_red());
                if !config.features.auto_reconnect {
                    return Err(e);
                }
                reconnect_attempts += 1;
                if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
                    return Err("Max reconnection attempts reached".into());
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        state.connected = true;
        println!("{}", "Connected!".bright_green());

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send nickname to server
        writer.write_all(nickname.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read OK from server
        let mut line = String::new();
        if reader.read_line(&mut line).await.is_err() {
            eprintln!(
                "{}",
                "Failed to establish connection with server.".bright_red()
            );
            if config.features.auto_reconnect {
                state.connected = false;
                continue;
            }
            return Err("Connection failed".into());
        }

        let (tx, mut rx) = mpsc::channel::<String>(100);
        let config_clone = config.clone();
        let state_clone = std::sync::Arc::new(tokio::sync::Mutex::new(state));

        // Spawn task to read from stdin
        let stdin_task = {
            let state = state_clone.clone();
            tokio::spawn(async move {
                let mut stdin = BufReader::new(tokio::io::stdin());
                let mut line = String::new();

                loop {
                    print!("> ");
                    io::stdout().flush().unwrap();
                    line.clear();
                    match stdin.read_line(&mut line).await {
                        Ok(0) => break,
                        Ok(_) => {
                            let msg = line.trim();
                            if !msg.is_empty() {
                                let mut state = state.lock().await;
                                state.add_to_history(msg.to_string());
                                drop(state);

                                if tx.send(msg.to_string()).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from stdin: {}", e);
                            break;
                        }
                    }
                }
            })
        };

        // Spawn task to read from server and print to stdout
        let server_read_task = {
            let config = config_clone.clone();
            let log_file_clone = log_file.take();
            tokio::spawn(async move {
                let mut reader = reader;
                let mut line = String::new();
                let mut log_file = log_file_clone;

                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => {
                            println!("\n{}", "[Server disconnected]".bright_red());
                            break;
                        }
                        Ok(_) => {
                            let formatted = format_message(&line, &config);
                            print!("{}", formatted);
                            io::stdout().flush().unwrap();

                            // Log to file if enabled
                            if let Some(ref mut file) = log_file {
                                use std::io::Write;
                                let _ = file.write_all(line.as_bytes());
                                let _ = file.flush();
                            }

                            // Play sound notification
                            if config.features.sound_notifications && !line.trim().is_empty() {
                                play_notification_sound();
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from server: {}", e);
                            break;
                        }
                    }
                }

                log_file
            })
        };

        // Main task: forward stdin messages to server
        let mut connection_lost = false;
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => {
                            // Send to server
                            if writer.write_all(msg.as_bytes()).await.is_err() {
                                connection_lost = true;
                                break;
                            }
                            if writer.write_all(b"\n").await.is_err() {
                                connection_lost = true;
                                break;
                            }
                            if writer.flush().await.is_err() {
                                connection_lost = true;
                                break;
                            }
                        }
                        None => break,
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("\n{}", "[Disconnecting...]".bright_yellow());
                    break;
                }
            }
        }

        stdin_task.abort();
        let log_file_result = server_read_task.await;
        if let Ok(file) = log_file_result {
            log_file = file;
        }

        if connection_lost && config.features.auto_reconnect {
            state = ClientState::new();
            println!(
                "{}",
                "Connection lost. Attempting to reconnect...".bright_yellow()
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            continue;
        } else {
            break;
        }
    }

    Ok(())
}
