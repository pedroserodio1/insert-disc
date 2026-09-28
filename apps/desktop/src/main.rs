// App desktop: janela Tauri sobre o mesmo `Host` do servidor de desenvolvimento.
// Por enquanto usa o drive falso (o WindowsDrive depende dos spikes W2-W5); o lançador só registra.
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

fn main() {
    let host: Shared = Arc::new(Mutex::new(Host::demo(std::env::temp_dir().join("insert-disc-demo"))));
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
