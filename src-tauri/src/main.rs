// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{params, Connection};
use anyhow::Result;

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

fn get_connections(conn: &Connection) -> Result<Vec<(i32, String, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, name, host, username FROM connections ORDER BY id DESC")?;
    let connections_iter = stmt.query_map([], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    })?;
    let mut connections = Vec::new();
    for connection in connections_iter {
        connections.push(connection?);  // Convert rusqlite::Error to anyhow::Error
    }
    Ok(connections)
}

fn get_connection(conn: &Connection) -> Result<Vec<(i32, String, String, String, String)>> {
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
        connections.push(connection?);  // Convert rusqlite::Error to anyhow::Error
    }
    Ok(connections)
}




#[tauri::command]
fn add_connection_command(name: String, host: String, username: String, password: String) -> Result<(), String> {
    let conn = initialize_db().map_err(|e| e.to_string())?;
    add_connection(&conn, &name, &host, &username, &password).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_connections_command() -> Result<Vec<(i32, String, String, String)>, String> {
    let conn = initialize_db().map_err(|e| e.to_string())?;
    let connections = get_connections(&conn).map_err(|e| e.to_string())?;
    Ok(connections)
}

#[tauri::command]
fn get_connection_command() -> Result<Vec<(i32, String, String, String, String)>, String> {
    let conn = initialize_db().map_err(|e| e.to_string())?;
    let connections = get_connection(&conn).map_err(|e| e.to_string())?;
    Ok(connections)
}

#[tauri::command]
fn delete_connection_command(id: i32) -> Result<(), String> {
    let conn = initialize_db().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM connections WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}



fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![add_connection_command, get_connections_command, get_connection_command, delete_connection_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}