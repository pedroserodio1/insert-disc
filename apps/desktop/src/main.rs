// App desktop: janela Tauri sobre o mesmo `Host` do servidor de desenvolvimento.
// Sem drive físico ainda (W2-W5): o drive falso só entra por opção; o lançador só registra.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use insert_disc_host::Host;
use serde_json::Value;
use tauri::State;

type Shared = Arc<Mutex<Host>>;

#[tauri::command]
fn snapshot(host: State<Shared>) -> String {
    host.lock().unwrap().snapshot_json()
}

#[tauri::command]
fn intent(host: State<Shared>, intent: Value) -> Result<Value, String> {
    host.lock().unwrap().intent(&intent)
}

#[tauri::command]
fn export_catalog(host: State<Shared>) -> String {
    host.lock().unwrap().export_json()
}

#[tauri::command]
fn import_catalog(host: State<Shared>, json: String) -> Result<(), String> {
    host.lock().unwrap().import_json(&json)
}

#[tauri::command]
fn dev_state(host: State<Shared>) -> Value {
    host.lock().unwrap().dev_state()
}

#[tauri::command]
fn dev(host: State<Shared>, cmd: Value) -> Result<Value, String> {
    host.lock().unwrap().dev(&cmd)
}

/// Pasta de dados do usuário: `%APPDATA%\InsertDisc` no Windows, `~/.local/share/insert-disc` no resto.
fn data_dir() -> std::path::PathBuf {
    if let Some(a) = std::env::var_os("APPDATA") {
        return std::path::PathBuf::from(a).join("InsertDisc");
    }
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir);
    home.join(".local/share/insert-disc")
}

/// `--demo`: estante de exemplo em pasta temporária (nunca toca os dados do usuário).
/// `--fake-drive`: catálogo real com o drive falso (padrão só em builds de depuração; Q7, SECURITY R8).
fn build_host() -> Host {
    let args: Vec<String> = std::env::args().collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    if has("--demo") {
        return Host::demo(std::env::temp_dir().join("insert-disc-demo"));
    }
    Host::open(data_dir(), has("--fake-drive") || cfg!(debug_assertions))
}

fn main() {
    let host: Shared = Arc::new(Mutex::new(build_host()));
    let ticker = host.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(50));
        ticker.lock().unwrap().tick();
    });

    tauri::Builder::default()
        .manage(host)
        .invoke_handler(tauri::generate_handler![snapshot, intent, export_catalog, import_catalog, dev_state, dev])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o app");
}
