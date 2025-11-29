use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};

type RoomId = String;
type ClientId = u64;

struct Room {
    id: RoomId,
    tx: broadcast::Sender<(ClientId, String, String)>, // (client_id, nickname, message)
    clients: Arc<RwLock<HashMap<ClientId, String>>>, // client_id -> nickname
}

struct ServerState {
    rooms: Arc<RwLock<HashMap<RoomId, Arc<Room>>>>,
    client_counter: Arc<RwLock<u64>>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            client_counter: Arc::new(RwLock::new(0)),
        }
    }

    async fn create_room(&self) -> RoomId {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let hash = timestamp % 1679616; // 36^4
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
            tx,
            clients: Arc::new(RwLock::new(HashMap::new())),
        });
        
        self.rooms.write().await.insert(room_id.clone(), room);
        room_id
    }

    async fn get_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.read().await.get(room_id).cloned()
    }

    async fn remove_room_if_empty(&self, room_id: &str) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            if room.clients.read().await.is_empty() {
                drop(rooms);
                self.rooms.write().await.remove(room_id);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Get bind address from command line or use default
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    let state = Arc::new(ServerState::new());

    loop {
        let (stream, addr) = listener.accept().await?;
        tracing::info!("New connection from {}", addr);

        let state = state.clone();
        let client_id = {
            let mut counter = state.client_counter.write().await;
            *counter += 1;
            *counter
        };

        tokio::spawn(async move {
            handle_client(stream, state, client_id).await;
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    state: Arc<ServerState>,
    client_id: ClientId,
) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut current_room: Option<Arc<Room>> = None;
    
    // Read nickname first
    line.clear();
    let nickname = match reader.read_line(&mut line).await {
        Ok(0) => return,
        Ok(_) => {
            let nick = line.trim().to_string();
            // Validate and sanitize nickname
            let nick = if nick.is_empty() {
                format!("user-{}", client_id)
            } else if nick.len() > 32 {
                // Limit nickname length
                nick.chars().take(32).collect()
            } else {
                nick
            };
            nick
        }
        Err(_) => return,
    };

    // Send welcome
    let _ = writer.write_all(b"OK\n").await;
    let _ = writer.flush().await;

    // Channel to send messages to writer task
    let (msg_tx, mut msg_rx) = tokio::sync::mpsc::channel::<String>(100);

    // Spawn task to write messages to client
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

    // Main loop: read commands and messages
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

                // Handle commands
                if input.starts_with('/') {
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    match parts[0] {
                        "/create" => {
                            let room_id = state.create_room().await;
                            let room = state.get_room(&room_id).await.unwrap();
                            
                            // Leave old room if any
                            if let Some(old_room) = &current_room {
                                old_room.clients.write().await.remove(&client_id);
                                state.remove_room_if_empty(&old_room.id).await;
                            }
                            
                            // Add client to new room
                            room.clients.write().await.insert(client_id, nickname.clone());
                            
                            // Spawn message receiver for this room
                            let mut rx = room.tx.subscribe();
                            let msg_tx_clone = msg_tx.clone();
                            let client_id_clone = client_id;
                            tokio::spawn(async move {
                                loop {
                                    match rx.recv().await {
                                        Ok((sender_id, nick, msg)) => {
                                            if sender_id != client_id_clone {
                                                // Format message with nickname (unless it's already a system message)
                                                let formatted = if msg.starts_with('[') {
                                                    format!("{}\n", msg)
                                                } else {
                                                    format!("{}: {}\n", nick, msg)
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
                            
                            let msg = format!("Room created: {}\nShare this ID with others to join.\n", room_id);
                            let _ = msg_tx.send(msg).await;

                            // Notify others in room
                            let join_msg = format!("[{} joined]", nickname);
                            let _ = room.tx.send((client_id, nickname.clone(), join_msg));
                        }
                        "/join" => {
                            if parts.len() < 2 {
                                let _ = msg_tx.send("Usage: /join <room-id>\n".to_string()).await;
                                continue;
                            }
                            
                            let room_id = parts[1].trim();
                            if room_id.is_empty() {
                                let _ = msg_tx.send("Error: Room ID cannot be empty\n".to_string()).await;
                                continue;
                            }
                            if let Some(room) = state.get_room(room_id).await {
                                // Leave current room if any
                                if let Some(old_room) = &current_room {
                                    old_room.clients.write().await.remove(&client_id);
                                    state.remove_room_if_empty(&old_room.id).await;
                                }
                                
                                // Join new room
                                room.clients.write().await.insert(client_id, nickname.clone());
                                
                                // Spawn message receiver for this room
                                let mut rx = room.tx.subscribe();
                                let msg_tx_clone = msg_tx.clone();
                                let client_id_clone = client_id;
                                tokio::spawn(async move {
                                    loop {
                                        match rx.recv().await {
                                            Ok((sender_id, nick, msg)) => {
                                                if sender_id != client_id_clone {
                                                    // Format message with nickname (unless it's already a system message)
                                                    let formatted = if msg.starts_with('[') {
                                                        format!("{}\n", msg)
                                                    } else {
                                                        format!("{}: {}\n", nick, msg)
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

                                // Notify others
                                let join_msg = format!("[{} joined]", nickname);
                                let _ = room.tx.send((client_id, nickname.clone(), join_msg));
                            } else {
                                let msg = format!("Room not found: {}\n", room_id);
                                let _ = msg_tx.send(msg).await;
                            }
                        }
                        "/quit" => {
                            if let Some(room) = &current_room {
                                room.clients.write().await.remove(&client_id);
                                state.remove_room_if_empty(&room.id).await;
                                current_room = None;
                                
                                let _ = msg_tx.send("Left the room.\n".to_string()).await;
                            } else {
                                let _ = msg_tx.send("You are not in any room.\n".to_string()).await;
                            }
                        }
                        "/help" => {
                            let help = "Commands:\n  /create       - Create a new room\n  /join <id>    - Join existing room\n  /quit         - Leave room\n  /help         - Show commands\n";
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
                    // Limit message length to prevent abuse
                    let message = if input.len() > 1000 {
                        input.chars().take(1000).collect::<String>()
                    } else {
                        input.to_string()
                    };
                    let _ = room.tx.send((client_id, nickname.clone(), message));
                } else {
                    let _ = msg_tx.send("You must join a room first. Use /create or /join <id>\n".to_string()).await;
                }
            }
            Err(e) => {
                tracing::error!("Error reading from client {}: {}", client_id, e);
                break;
            }
        }
    }

    // Cleanup: remove from room on disconnect
    if let Some(room) = &current_room {
        room.clients.write().await.remove(&client_id);
        state.remove_room_if_empty(&room.id).await;
    }
    
    drop(msg_tx);
    let _ = write_task.await;
}
