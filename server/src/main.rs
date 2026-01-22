use chrono::{DateTime, Local};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;

type RoomId = String;
type ClientId = u64;

#[derive(Clone, Debug, PartialEq)]
enum RoomType {
    Public,
    Private,
}

#[derive(Clone, Debug)]
struct Message {
    timestamp: DateTime<Local>,
    #[allow(dead_code)]
    client_id: ClientId, // Reserved for future moderation/logging features
    nickname: String,
    content: String,
    is_action: bool,
}

#[derive(Clone, Debug)]
struct Room {
    id: RoomId,
    room_type: RoomType,
    password: Option<String>,
    capacity: Option<u32>,
    tx: broadcast::Sender<(ClientId, String, String, bool)>, // (client_id, nickname, message, is_action)
    clients: Arc<RwLock<HashMap<ClientId, String>>>,         // client_id -> nickname
    admins: Arc<RwLock<HashSet<ClientId>>>,
    moderators: Arc<RwLock<HashSet<ClientId>>>,
    banned: Arc<RwLock<HashSet<ClientId>>>,
    message_history: Arc<RwLock<Vec<Message>>>,
    #[allow(dead_code)]
    created_at: DateTime<Local>, // Reserved for future room age/analytics features
    persistent: bool,
}

struct ClientInfo {
    #[allow(dead_code)]
    id: ClientId, // Reserved for future logging/analytics features
    #[allow(dead_code)]
    nickname: Arc<RwLock<String>>, // Stored for consistency, accessed via separate nickname_arc
    #[allow(dead_code)]
    last_message_time: Arc<RwLock<Instant>>, // Reserved for future rate limiting enhancements
    message_count: Arc<RwLock<u32>>,
    rate_limit_window: Arc<RwLock<Instant>>,
    #[allow(dead_code)]
    ip: String, // Reserved for future moderation/logging features
}

struct ServerState {
    rooms: Arc<RwLock<HashMap<RoomId, Arc<Room>>>>,
    clients: Arc<RwLock<HashMap<ClientId, Arc<ClientInfo>>>>,
    client_counter: Arc<RwLock<u64>>,
    total_users: Arc<RwLock<u64>>,
    active_users: Arc<RwLock<u32>>,
    total_messages: Arc<RwLock<u64>>,
    start_time: Instant,
}

impl ServerState {
    fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            client_counter: Arc::new(RwLock::new(0)),
            total_users: Arc::new(RwLock::new(0)),
            active_users: Arc::new(RwLock::new(0)),
            total_messages: Arc::new(RwLock::new(0)),
            start_time: Instant::now(),
        }
    }

    async fn create_room(
        &self,
        room_type: RoomType,
        password: Option<String>,
        capacity: Option<u32>,
        persistent: bool,
    ) -> RoomId {
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let hash = timestamp % 1679616;
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
        let mut id_part = String::new();
        let mut n = hash;
        for _ in 0..5 {
            id_part.push(chars[(n % 36) as usize]);
            n /= 36;
        }
        let room_id = format!("room-{}", id_part);

        let (tx, _rx) = broadcast::channel(1000);
        let room = Arc::new(Room {
            id: room_id.clone(),
            room_type,
            password,
            capacity,
            tx,
            clients: Arc::new(RwLock::new(HashMap::new())),
            admins: Arc::new(RwLock::new(HashSet::new())),
            moderators: Arc::new(RwLock::new(HashSet::new())),
            banned: Arc::new(RwLock::new(HashSet::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
            created_at: Local::now(),
            persistent,
        });

        self.rooms.write().await.insert(room_id.clone(), room);
        room_id
    }

    async fn get_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.read().await.get(room_id).cloned()
    }

    async fn list_public_rooms(&self) -> Vec<(RoomId, u32, u32)> {
        let rooms = self.rooms.read().await;
        let mut result = Vec::new();
        for (id, room) in rooms.iter() {
            if room.room_type == RoomType::Public {
                let client_count = room.clients.read().await.len() as u32;
                let capacity = room.capacity.unwrap_or(0);
                result.push((id.clone(), client_count, capacity));
            }
        }
        result
    }

    async fn remove_room_if_empty(&self, room_id: &str) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            if !room.persistent && room.clients.read().await.is_empty() {
                drop(rooms);
                self.rooms.write().await.remove(room_id);
            }
        }
    }

    fn sanitize_input(input: &str) -> String {
        input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r')
            .take(1000)
            .collect()
    }

    async fn check_rate_limit(client: &Arc<ClientInfo>) -> bool {
        let now = Instant::now();
        let mut window = client.rate_limit_window.write().await;
        let mut count = client.message_count.write().await;

        if now.duration_since(*window) > Duration::from_secs(1) {
            *window = now;
            *count = 0;
        }

        *count += 1;
        *count <= 10 // 10 messages per second
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    let state = Arc::new(ServerState::new());

    // Spawn metrics display task
    let state_metrics = state.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let total_users = *state_metrics.total_users.read().await;
            let active_users = *state_metrics.active_users.read().await;
            let total_messages = *state_metrics.total_messages.read().await;
            let total_rooms = state_metrics.rooms.read().await.len();
            let uptime = state_metrics.start_time.elapsed().as_secs();

            print!("\r\x1b[K"); // Clear line
            print!(
                "Metrics | Users: {} (Active: {}) | Messages: {} | Rooms: {} | Uptime: {}s",
                total_users, active_users, total_messages, total_rooms, uptime
            );
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    });

    // Graceful shutdown handler
    let _state_shutdown = state.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("\n\nShutting down gracefully...");
        // Save persistent rooms here if needed
        std::process::exit(0);
    });

    loop {
        let (stream, addr) = listener.accept().await?;
        let ip = addr.ip().to_string();
        tracing::info!("New connection from {}", addr);

        let state = state.clone();
        let client_id = {
            let mut counter = state.client_counter.write().await;
            *counter += 1;
            *counter
        };

        {
            let mut total = state.total_users.write().await;
            *total += 1;
        }
        {
            let mut active = state.active_users.write().await;
            *active += 1;
        }

        let state_clone = state.clone();
        tokio::spawn(async move {
            handle_client(stream, state, client_id, ip).await;
            let mut active = state_clone.active_users.write().await;
            *active = active.saturating_sub(1);
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    state: Arc<ServerState>,
    client_id: ClientId,
    ip: String,
) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut current_room: Option<Arc<Room>> = None;

    line.clear();
    let initial_nickname = match reader.read_line(&mut line).await {
        Ok(0) => return,
        Ok(_) => {
            let nick = line.trim().to_string();
            let nick = if nick.is_empty() {
                format!("user-{}", client_id)
            } else if nick.len() > 32 {
                nick.chars().take(32).collect()
            } else {
                ServerState::sanitize_input(&nick)
            };
            nick
        }
        Err(_) => return,
    };

    let nickname_arc = Arc::new(RwLock::new(initial_nickname.clone()));
    let client_info = Arc::new(ClientInfo {
        id: client_id,
        nickname: nickname_arc.clone(),
        last_message_time: Arc::new(RwLock::new(Instant::now())),
        message_count: Arc::new(RwLock::new(0)),
        rate_limit_window: Arc::new(RwLock::new(Instant::now())),
        ip: ip.clone(),
    });

    state
        .clients
        .write()
        .await
        .insert(client_id, client_info.clone());

    let _ = writer.write_all(b"OK\n").await;
    let _ = writer.flush().await;

    let (msg_tx, mut msg_rx) = tokio::sync::mpsc::channel::<String>(100);

    // Helper to get current nickname
    let get_nickname = || async { nickname_arc.read().await.clone() };

    let write_task = tokio::spawn(async move {
        loop {
            match msg_rx.recv().await {
                Some(msg) => {
                    if writer.write_all(msg.as_bytes()).await.is_err() {
                        break;
                    }
                    if writer.flush().await.is_err() {
                        break;
                    }
                }
                None => break,
            }
        }
    });

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                tracing::info!("Client {} disconnected", client_id);
                break;
            }
            Ok(_) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                // Rate limiting
                if !ServerState::check_rate_limit(&client_info).await {
                    let _ = msg_tx
                        .send("Rate limit exceeded. Please slow down.\n".to_string())
                        .await;
                    continue;
                }

                let sanitized = ServerState::sanitize_input(input);

                if sanitized.starts_with('/') {
                    let parts: Vec<&str> = sanitized.split_whitespace().collect();
                    match parts[0] {
                        "/create" | "/c" => {
                            let room_type = if parts.len() > 1 && parts[1] == "private" {
                                RoomType::Private
                            } else {
                                RoomType::Public
                            };

                            let password = parts
                                .iter()
                                .position(|&p| p == "password" || p == "pwd")
                                .and_then(|idx| parts.get(idx + 1))
                                .map(|s| s.to_string());

                            let capacity = parts
                                .iter()
                                .position(|&p| p == "capacity" || p == "cap")
                                .and_then(|idx| parts.get(idx + 1))
                                .and_then(|s| s.parse::<u32>().ok());

                            let persistent =
                                parts.contains(&"persistent") || parts.contains(&"perm");

                            let room_id = state
                                .create_room(
                                    room_type.clone(),
                                    password.clone(),
                                    capacity,
                                    persistent,
                                )
                                .await;
                            let room = state.get_room(&room_id).await.unwrap();

                            if let Some(old_room) = &current_room {
                                old_room.clients.write().await.remove(&client_id);
                                state.remove_room_if_empty(&old_room.id).await;
                            }

                            let current_nick = get_nickname().await;
                            room.clients
                                .write()
                                .await
                                .insert(client_id, current_nick.clone());
                            room.admins.write().await.insert(client_id);

                            let mut rx = room.tx.subscribe();
                            let msg_tx_clone = msg_tx.clone();
                            let client_id_clone = client_id;
                            tokio::spawn(async move {
                                loop {
                                    match rx.recv().await {
                                        Ok((sender_id, nick, msg, is_action)) => {
                                            if sender_id != client_id_clone {
                                                let timestamp = Local::now().format("%H:%M");
                                                let formatted = if msg.starts_with('[') {
                                                    format!("[{}] {}\n", timestamp, msg)
                                                } else if is_action {
                                                    format!("[{}] * {} {}\n", timestamp, nick, msg)
                                                } else {
                                                    format!("[{}] {}: {}\n", timestamp, nick, msg)
                                                };
                                                if msg_tx_clone.send(formatted).await.is_err() {
                                                    break;
                                                }
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            });

                            current_room = Some(room.clone());

                            let mut msg = format!(
                                "Room created: {} ({})\n",
                                room_id,
                                if room_type == RoomType::Public {
                                    "public"
                                } else {
                                    "private"
                                }
                            );
                            if password.is_some() {
                                msg.push_str("Password protected\n");
                            }
                            if capacity.is_some() {
                                msg.push_str(&format!("Capacity: {}\n", capacity.unwrap()));
                            }
                            msg.push_str("Share this ID with others to join.\n");
                            let _ = msg_tx.send(msg).await;

                            let current_nick = get_nickname().await;
                            let join_msg = format!("[{} joined]", current_nick);
                            let _ = room.tx.send((client_id, current_nick, join_msg, false));
                        }
                        "/join" | "/j" => {
                            if parts.len() < 2 {
                                let _ = msg_tx
                                    .send("Usage: /join <room-id> [password]\n".to_string())
                                    .await;
                                continue;
                            }

                            let room_id = parts[1].trim();
                            let provided_password = parts.get(2).map(|s| s.to_string());

                            if let Some(room) = state.get_room(room_id).await {
                                if room.banned.read().await.contains(&client_id) {
                                    let _ = msg_tx
                                        .send("You are banned from this room.\n".to_string())
                                        .await;
                                    continue;
                                }

                                if let Some(room_password) = &room.password {
                                    if provided_password.as_ref().map(|s| s.as_str())
                                        != Some(room_password)
                                    {
                                        let _ =
                                            msg_tx.send("Incorrect password.\n".to_string()).await;
                                        continue;
                                    }
                                }

                                if let Some(cap) = room.capacity {
                                    if room.clients.read().await.len() >= cap as usize {
                                        let _ = msg_tx.send("Room is full.\n".to_string()).await;
                                        continue;
                                    }
                                }

                                if let Some(old_room) = &current_room {
                                    old_room.clients.write().await.remove(&client_id);
                                    state.remove_room_if_empty(&old_room.id).await;
                                }

                                let current_nick = get_nickname().await;
                                room.clients
                                    .write()
                                    .await
                                    .insert(client_id, current_nick.clone());

                                // Send message history
                                let history = room.message_history.read().await;
                                let recent_history: Vec<String> = history
                                    .iter()
                                    .rev()
                                    .take(50)
                                    .rev()
                                    .map(|m| {
                                        let timestamp = m.timestamp.format("%H:%M");
                                        if m.is_action {
                                            format!(
                                                "[{}] * {} {}\n",
                                                timestamp, m.nickname, m.content
                                            )
                                        } else {
                                            format!(
                                                "[{}] {}: {}\n",
                                                timestamp, m.nickname, m.content
                                            )
                                        }
                                    })
                                    .collect();
                                for hist_msg in recent_history {
                                    let _ = msg_tx.send(hist_msg).await;
                                }
                                drop(history);

                                let mut rx = room.tx.subscribe();
                                let msg_tx_clone = msg_tx.clone();
                                let client_id_clone = client_id;
                                tokio::spawn(async move {
                                    loop {
                                        match rx.recv().await {
                                            Ok((sender_id, nick, msg, is_action)) => {
                                                if sender_id != client_id_clone {
                                                    let timestamp = Local::now().format("%H:%M");
                                                    let formatted = if msg.starts_with('[') {
                                                        format!("[{}] {}\n", timestamp, msg)
                                                    } else if is_action {
                                                        format!(
                                                            "[{}] * {} {}\n",
                                                            timestamp, nick, msg
                                                        )
                                                    } else {
                                                        format!(
                                                            "[{}] {}: {}\n",
                                                            timestamp, nick, msg
                                                        )
                                                    };
                                                    if msg_tx_clone.send(formatted).await.is_err() {
                                                        break;
                                                    }
                                                }
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                });

                                current_room = Some(room.clone());

                                let msg = format!("Joined room: {}\n", room_id);
                                let _ = msg_tx.send(msg).await;

                                let current_nick = get_nickname().await;
                                let join_msg = format!("[{} joined]", current_nick);
                                let _ = room.tx.send((client_id, current_nick, join_msg, false));
                            } else {
                                let msg = format!("Room not found: {}\n", room_id);
                                let _ = msg_tx.send(msg).await;
                            }
                        }
                        "/list" | "/l" => {
                            let public_rooms = state.list_public_rooms().await;
                            if public_rooms.is_empty() {
                                let _ = msg_tx
                                    .send("No public rooms available.\n".to_string())
                                    .await;
                            } else {
                                let mut list = "Public rooms:\n".to_string();
                                for (id, users, cap) in public_rooms {
                                    if cap > 0 {
                                        list.push_str(&format!(
                                            "  {} - {}/{} users\n",
                                            id, users, cap
                                        ));
                                    } else {
                                        list.push_str(&format!("  {} - {} users\n", id, users));
                                    }
                                }
                                let _ = msg_tx.send(list).await;
                            }
                        }
                        "/users" | "/u" => {
                            if let Some(room) = &current_room {
                                let clients = room.clients.read().await;
                                let admins = room.admins.read().await;
                                let mods = room.moderators.read().await;

                                let mut user_list = format!("Users in room ({}):\n", clients.len());
                                for (id, nick) in clients.iter() {
                                    let role = if admins.contains(id) {
                                        " [Admin]"
                                    } else if mods.contains(id) {
                                        " [Mod]"
                                    } else {
                                        ""
                                    };
                                    user_list.push_str(&format!("  {}{}\n", nick, role));
                                }
                                let _ = msg_tx.send(user_list).await;
                            } else {
                                let _ = msg_tx.send("You are not in any room.\n".to_string()).await;
                            }
                        }
                        "/me" => {
                            if parts.len() < 2 {
                                let _ = msg_tx.send("Usage: /me <action>\n".to_string()).await;
                                continue;
                            }
                            if let Some(room) = &current_room {
                                let action = parts[1..].join(" ");
                                let sanitized_action = ServerState::sanitize_input(&action);

                                let message = Message {
                                    timestamp: Local::now(),
                                    client_id,
                                    nickname: get_nickname().await.clone(),
                                    content: sanitized_action.clone(),
                                    is_action: true,
                                };
                                room.message_history.write().await.push(message);

                                let mut total = state.total_messages.write().await;
                                *total += 1;

                                let current_nick = get_nickname().await;
                                let _ =
                                    room.tx
                                        .send((client_id, current_nick, sanitized_action, true));
                            } else {
                                let _ = msg_tx
                                    .send("You must join a room first.\n".to_string())
                                    .await;
                            }
                        }
                        "/kick" => {
                            if parts.len() < 2 {
                                let _ = msg_tx.send("Usage: /kick <nickname>\n".to_string()).await;
                                continue;
                            }
                            if let Some(room) = &current_room {
                                let is_admin = room.admins.read().await.contains(&client_id);
                                let is_mod = room.moderators.read().await.contains(&client_id);

                                if !is_admin && !is_mod {
                                    let _ = msg_tx
                                        .send(
                                            "You don't have permission to kick users.\n"
                                                .to_string(),
                                        )
                                        .await;
                                    continue;
                                }

                                let target_nick = parts[1];
                                let clients = room.clients.read().await;
                                let target_id = clients
                                    .iter()
                                    .find(|(_, nick)| *nick == target_nick)
                                    .map(|(id, _)| *id);

                                if let Some(target) = target_id {
                                    if target == client_id {
                                        let _ = msg_tx
                                            .send("You cannot kick yourself.\n".to_string())
                                            .await;
                                        continue;
                                    }

                                    let target_is_admin =
                                        room.admins.read().await.contains(&target);
                                    if target_is_admin && !is_admin {
                                        let _ = msg_tx
                                            .send("You cannot kick an admin.\n".to_string())
                                            .await;
                                        continue;
                                    }

                                    room.clients.write().await.remove(&target);
                                    let kick_msg = format!("[{} was kicked]", target_nick);
                                    let current_nick = get_nickname().await;
                                    let _ =
                                        room.tx.send((client_id, current_nick, kick_msg, false));
                                    let _ = msg_tx
                                        .send(format!(
                                            "{} was kicked from the room.\n",
                                            target_nick
                                        ))
                                        .await;
                                } else {
                                    let _ = msg_tx
                                        .send(format!(
                                            "User '{}' not found in room.\n",
                                            target_nick
                                        ))
                                        .await;
                                }
                            } else {
                                let _ = msg_tx.send("You are not in any room.\n".to_string()).await;
                            }
                        }
                        "/ban" => {
                            if parts.len() < 2 {
                                let _ = msg_tx.send("Usage: /ban <nickname>\n".to_string()).await;
                                continue;
                            }
                            if let Some(room) = &current_room {
                                if !room.admins.read().await.contains(&client_id) {
                                    let _ = msg_tx
                                        .send("Only admins can ban users.\n".to_string())
                                        .await;
                                    continue;
                                }

                                let target_nick = parts[1];
                                let clients = room.clients.read().await;
                                let target_id = clients
                                    .iter()
                                    .find(|(_, nick)| *nick == target_nick)
                                    .map(|(id, _)| *id);

                                if let Some(target) = target_id {
                                    if target == client_id {
                                        let _ = msg_tx
                                            .send("You cannot ban yourself.\n".to_string())
                                            .await;
                                        continue;
                                    }

                                    room.clients.write().await.remove(&target);
                                    room.banned.write().await.insert(target);
                                    let ban_msg = format!("[{} was banned]", target_nick);
                                    let current_nick = get_nickname().await;
                                    let _ = room.tx.send((client_id, current_nick, ban_msg, false));
                                    let _ = msg_tx
                                        .send(format!(
                                            "{} was banned from the room.\n",
                                            target_nick
                                        ))
                                        .await;
                                } else {
                                    let _ = msg_tx
                                        .send(format!(
                                            "User '{}' not found in room.\n",
                                            target_nick
                                        ))
                                        .await;
                                }
                            } else {
                                let _ = msg_tx.send("You are not in any room.\n".to_string()).await;
                            }
                        }
                        "/nick" => {
                            if parts.len() < 2 {
                                let _ = msg_tx
                                    .send("Usage: /nick <new-nickname>\n".to_string())
                                    .await;
                                continue;
                            }
                            let new_nick = ServerState::sanitize_input(parts[1]);
                            if new_nick.len() > 32 {
                                let _ = msg_tx
                                    .send("Nickname too long (max 32 characters).\n".to_string())
                                    .await;
                                continue;
                            }

                            // Update nickname in all rooms
                            if let Some(room) = &current_room {
                                if let Some(old_nick) = room
                                    .clients
                                    .write()
                                    .await
                                    .insert(client_id, new_nick.clone())
                                {
                                    let change_msg =
                                        format!("[{} changed name to {}]", old_nick, new_nick);
                                    let _ = room.tx.send((
                                        client_id,
                                        new_nick.clone(),
                                        change_msg,
                                        false,
                                    ));
                                }
                            }

                            *nickname_arc.write().await = new_nick.clone();
                            let _ = msg_tx
                                .send(format!("Nickname changed to: {}\n", new_nick))
                                .await;
                        }
                        "/quit" | "/q" => {
                            if let Some(room) = &current_room {
                                room.clients.write().await.remove(&client_id);
                                state.remove_room_if_empty(&room.id).await;
                                current_room = None;

                                let _ = msg_tx.send("Left the room.\n".to_string()).await;
                            } else {
                                let _ = msg_tx.send("You are not in any room.\n".to_string()).await;
                            }
                        }
                        "/help" | "/h" => {
                            let help = "Commands:\n  /create [public|private] [password <pwd>] [capacity <n>] [persistent] - Create room\n  /join <id> [password] - Join room\n  /list - List public rooms\n  /users - Show room users\n  /me <action> - Action message\n  /kick <user> - Kick user (mod/admin)\n  /ban <user> - Ban user (admin)\n  /nick <name> - Change nickname\n  /quit - Leave room\n  /help - Show commands\n";
                            let _ = msg_tx.send(help.to_string()).await;
                        }
                        _ => {
                            let msg = format!("Unknown command: {}\n", parts[0]);
                            let _ = msg_tx.send(msg).await;
                        }
                    }
                    continue;
                }

                // Handle regular messages
                if let Some(room) = &current_room {
                    let sanitized_msg = ServerState::sanitize_input(&sanitized);

                    let message = Message {
                        timestamp: Local::now(),
                        client_id,
                        nickname: get_nickname().await.clone(),
                        content: sanitized_msg.clone(),
                        is_action: false,
                    };
                    room.message_history.write().await.push(message);

                    // Keep only last 1000 messages
                    let mut history = room.message_history.write().await;
                    if history.len() > 1000 {
                        history.remove(0);
                    }
                    drop(history);

                    let mut total = state.total_messages.write().await;
                    *total += 1;

                    let current_nick = get_nickname().await;
                    let _ = room
                        .tx
                        .send((client_id, current_nick, sanitized_msg, false));
                } else {
                    let _ = msg_tx
                        .send("You must join a room first. Use /create or /join <id>\n".to_string())
                        .await;
                }
            }
            Err(e) => {
                tracing::error!("Error reading from client {}: {}", client_id, e);
                break;
            }
        }
    }

    if let Some(room) = &current_room {
        room.clients.write().await.remove(&client_id);
        state.remove_room_if_empty(&room.id).await;
    }

    state.clients.write().await.remove(&client_id);
    drop(msg_tx);
    let _ = write_task.await;
}
