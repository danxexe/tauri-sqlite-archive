#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{Connection, Result};
use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Default)]
struct DB(Mutex<Option<Connection>>);

fn init_db() -> DB {
    let conn = Connection::open("database.db3").unwrap();
    DB(Mutex::new(Some(conn)))
}

fn get_content(db: State<DB>, path: &str) -> Option<String> {
    db.0.lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .query_row("SELECT data FROM sqlar WHERE name = ?1", [path], |row| {
            row.get(0)
        })
        .ok()
}

fn main() -> Result<()> {
    let db = init_db();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(db)
        .register_uri_scheme_protocol("sqlar", |ctx, req| {
            let uri = req.uri();
            let db = ctx.app_handle().try_state::<DB>().unwrap();

            let response = if let Some(content) = get_content(db, uri.path()) {
                println!("200: {:?}", uri.path());
                http::Response::builder()
                    .status(200)
                    .header("Access-Control-Allow-Origin", "tauri://localhost")
                    .header("Content-Type", 
                        mime_guess::from_path(uri.path())
                            .first()
                            .unwrap()
                            .essence_str(),
                    )
                    .body(content.into())
                    .unwrap()
            } else {
                println!("404: {:?}", uri.path());
                http::Response::builder()
                    .status(302)
                    .header(
                        "Location",
                        format!("https://tauri.localhost/{}", uri.path()),
                    )
                    .body(vec![])
                    .unwrap()
            };

            let _ = ctx.app_handle().get_webview_window("main").unwrap().show();

            response
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
