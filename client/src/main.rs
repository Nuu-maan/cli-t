use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get server address from command line or use default
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    // Welcome message
    println!("Welcome to cli-t!\n");
    println!("Connecting to: {}\n", addr);
    
    // Get nickname
    print!("Nick (leave blank for random): ");
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
    
    println!("You are: {}\n", nickname);
    println!("Commands:");
    println!("  /create       - Create a new room");
    println!("  /join <id>    - Join existing room");
    println!("  /quit         - Leave room");
    println!("  /help         - Show commands\n");

    // Connect to server
    let stream = match TcpStream::connect(&addr).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to server at {}: {}", addr, e);
            eprintln!("Make sure the server is running.");
            return Err(e.into());
        }
    };

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send nickname to server
    writer.write_all(nickname.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // Read OK from server
    let mut line = String::new();
    if reader.read_line(&mut line).await.is_err() {
        eprintln!("Failed to establish connection with server.");
        return Err("Connection failed".into());
    }

    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Spawn task to read from stdin
    let stdin_task = tokio::spawn(async move {
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
    });

    // Spawn task to read from server and print to stdout
    let server_read_task = tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("\n[Server disconnected]");
                    break;
                }
                Ok(_) => {
                    print!("{}", line);
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    });

    // Main task: forward stdin messages to server
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(msg) => {
                        // Send to server
                        if writer.write_all(msg.as_bytes()).await.is_err() {
                            break;
                        }
                        if writer.write_all(b"\n").await.is_err() {
                            break;
                        }
                        if writer.flush().await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n[Disconnecting...]");
                break;
            }
        }
    }

    stdin_task.abort();
    server_read_task.abort();

    Ok(())
}
