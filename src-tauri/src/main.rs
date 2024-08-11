#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use rusqlite::{Connection, Result};
use tauri::{Manager, State};
use http::Uri;


#[derive(Default)]
struct DB(Mutex<Option<Connection>>);

fn init_db() -> DB {
    let conn = Connection::open("database.db3").unwrap();
    DB(Mutex::new(Some(conn)))
}

#[tauri::command]
fn get_content(db: State<DB>, path: &str) -> Option<String> {
    db.0.lock().unwrap().as_ref().unwrap().query_row(
        "SELECT data FROM sqlar WHERE name = ?1",
        [path],
        |row| row.get(0),
    ).ok()
}

fn main() -> Result<()> {
    let db = init_db();

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![get_content])
        .on_page_load(|window, _| {
            window.show().unwrap();
        })
        .register_uri_scheme_protocol("sqlar", |app, req| {
            let uri = req.uri().parse::<Uri>().unwrap();
            println!("{:?}", uri.path());

            let db = app.try_state::<DB>().unwrap();

            if let Some(content) = get_content(db, uri.path()) {
                tauri::http::ResponseBuilder::new()
                    .status(200)
                    .header("Access-Control-Allow-Origin", "tauri://localhost")
                    .mimetype(mime_guess::from_path(uri.path()).first().unwrap().essence_str())
                    .body(content.into())
            } else {
                tauri::http::ResponseBuilder::new()
                    .status(302)
                    .header("Location", format!("https://tauri.localhost/{}", uri.path()))
                    .body(vec![])
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

        Ok(())
}
