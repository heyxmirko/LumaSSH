// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


// MAIN LIBS
use rusqlite::{params, Connection};
use anyhow::Result;

// TERMINAL LIBS
use ssh2::Session;
use std::net::TcpStream;
use tauri::{Manager, State, Window};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use std::io::{Read, Write};


// -------------------------------------------------- MAIN FUNCTIONS - START --------------------------------------------------
fn initialize_db() -> Result<Connection> {
    let conn = Connection::open("connections.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS connections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            username TEXT NOT NULL,
            password TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

fn add_connection(conn: &Connection, name: &str, host: &str, username: &str, password: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO connections (name, host, username, password) VALUES (?1, ?2, ?3, ?4)",
        params![name, host, username, password],
    )?;
    Ok(())
}

fn get_connections(conn: &Connection) -> Result<Vec<(i32, String, String, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, name, host, username, password FROM connections ORDER BY id DESC")?;
    let connections_iter = stmt.query_map([], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?
        ))
    })?;
    let mut connections = Vec::new();
    for connection in connections_iter {
        connections.push(connection?);
    }
    Ok(connections)
}

fn get_connection(conn: &Connection, id: i32) -> Result<(String, String, String), String> {
    let mut stmt = conn.prepare("SELECT host, username, password FROM connections WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row([id], |row| {
        Ok((
            row.get(0)?,  // host
            row.get(1)?,  // username
            row.get(2)?,  // password
        ))
    }).map_err(|e| e.to_string());

    result
}



#[tauri::command]
fn add_connection_command(name: String, host: String, username: String, password: String) -> Result<(), String> {
    let conn = initialize_db().map_err(|e| e.to_string())?;
    add_connection(&conn, &name, &host, &username, &password).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_connections_command() -> Result<Vec<(i32, String, String, String, String)>, String> {
    let conn = initialize_db().map_err(|e| e.to_string())?;
    let connections = get_connections(&conn).map_err(|e| e.to_string())?;
    Ok(connections)
}

#[tauri::command]
fn get_connection_command(id: i32) -> Result<(String, String, String), String> {
    let conn = initialize_db().map_err(|e| e.to_string())?;
    let connections = get_connection(&conn, id).map_err(|e| e.to_string())?;
    Ok(connections)
}

#[tauri::command]
fn delete_connection_command(id: i32) -> Result<(), String> {
    let conn = initialize_db().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM connections WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
// -------------------------------------------------- MAIN FUNCTIONS - END ---------------------------------------------------- 







// -------------------------------------------------- TERMINAL FUNCTIONS - START --------------------------------------------------
type SSHState = Arc<Mutex<Option<Sender<String>>>>;

fn start_ssh_session(
    window: Window,
    ssh_state: SSHState,
    host: String,
    username: String,
    password: String,
) {
    // Establish SSH connection
    let tcp = match TcpStream::connect(&host) {
        Ok(tcp) => tcp,
        Err(err) => {
            window.emit("ssh_error", format!("Connection error: {}", err)).unwrap();
            return;
        }
    };

    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake().unwrap();

    // Authentication
    if let Err(err) = session.userauth_password(&username, &password) {
        window
            .emit("ssh_error", format!("Authentication failed: {}", err))
            .unwrap();
        return;
    }

    // Open a channel and request a pseudo-terminal (PTY)
    let mut ssh_channel = match session.channel_session() {
        Ok(channel) => channel,
        Err(err) => {
            window.emit("ssh_error", format!("Channel error: {}", err)).unwrap();
            return;
        }
    };

    // Request a PTY with default settings (skip terminal modes)
    if let Err(err) = ssh_channel.request_pty("xterm", None, None) {
        window
            .emit("ssh_error", format!("PTY request failed: {}", err))
            .unwrap();
        return;
    }

    // Start the shell
    if let Err(err) = ssh_channel.shell() {
        window
            .emit("ssh_error", format!("Failed to start shell: {}", err))
            .unwrap();
        return;
    }

    session.set_blocking(false);

    println!("Interactive shell started"); // Add this debugging line

    // Create an mpsc channel to receive input data
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    // Store the sender in the shared state
    let mut ssh_state = ssh_state.lock().unwrap();
    *ssh_state = Some(tx.clone());

    // Start a thread that handles the SSH session
    let window_clone = window.clone();
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            println!("Checking for SSH output...");
            
            loop {
                match ssh_channel.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                        println!("SSH output received: {}", output);  // Debugging output
                        window_clone.emit("ssh_output", output).unwrap();
                    }
                    Ok(_) => {
                        println!("No more data from SSH...");
                        break;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No data available for now, continue
                        println!("Would block, no more data right now");
                        break;
                    }
                    Err(err) => {
                        eprintln!("Read error: {:?}", err);
                        break;
                    }
                }
            }
        
            // Handle input from the user
            match rx.try_recv() {
                Ok(mut input) => {
                    if input.ends_with("\r") || input.ends_with("\n") {
                        input = input.trim_end().to_string() + "\r\n";
                    }
            
                    // Write input to the SSH channel
                    if let Err(err) = ssh_channel.write_all(input.as_bytes()) {
                        eprintln!("Write error: {:?}", err);
                        break;
                    }
            
                    // Ensure flushing after writing input
                    if let Err(err) = ssh_channel.flush() {
                        eprintln!("Flush error: {:?}", err);
                        break;
                    }
            
                    println!("Command sent to SSH: '{}'", input);  // Debugging line to confirm input sent
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // No input at the moment
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    eprintln!("Input channel disconnected");
                    break;
                }
            }
        
            // Sleep to prevent busy-waiting
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    
        // Close the SSH channel when done
        ssh_channel.close().unwrap();
        ssh_channel.wait_close().unwrap();
        println!("Channel closed");
    });
}

#[tauri::command]
fn start_ssh_session_command(
    host: String,
    username: String,
    password: String,
    window: Window,
    ssh_state: State<SSHState>,
) {
    // Clone the inner SSHState
    let ssh_state_clone = ssh_state.inner().clone();
    thread::spawn(move || {
        start_ssh_session(window, ssh_state_clone, host, username, password);
    });
}


#[tauri::command]
fn send_input(input: String, ssh_state: State<SSHState>) {
    println!("Received input: {}", input);
    let ssh_state = ssh_state.lock().unwrap();
    if let Some(ref tx) = *ssh_state {
        println!("Passed if check 1!");
        match tx.send(input) {
            Ok(_) => {
                println!("Input sent successfully");
            }
            Err(err) => {
                println!("Passed 2");
                eprintln!("Failed to send input: {:?}", err);
                println!("Passed 3");
            }
        }
    } else {
        eprintln!("SSH session is not initialized");
    }
}

// -------------------------------------------------- TERMINAL FUNCTIONS - END ----------------------------------------------------




fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let ssh_state: SSHState = Arc::new(Mutex::new(None));
            app.manage(ssh_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_connection_command,
            get_connections_command,
            delete_connection_command,
            start_ssh_session_command,
            get_connection_command,
            send_input
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}