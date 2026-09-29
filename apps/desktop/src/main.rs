// App desktop: janela Tauri sobre o mesmo `Host` do servidor de desenvolvimento.
// Sem drive físico ainda (W2-W5): o drive falso só entra por opção; o lançador só registra.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use insert_disc_host::{DriveMode, Host};
use serde_json::Value;
use tauri::{Manager, State};

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
fn set_cover(host: State<Shared>, request: Value) -> Result<Value, String> {
    host.lock().unwrap().set_cover_json(&request)
}

#[tauri::command]
fn steam_games(host: State<Shared>) -> Value {
    host.lock().unwrap().steam_games()
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
/// `--fake-drive` / `--real-drive`: drive falso (ISO) ou o do Windows. Padrão: falso em depuração,
/// Windows em release (Q7, SECURITY R8).
/// `--real-launch` / `--log-launch`: abre jogos de verdade ou só registra (padrão: real em release, log em depuração).
fn build_host() -> Host {
    let args: Vec<String> = std::env::args().collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    if has("--demo") {
        return Host::demo(std::env::temp_dir().join("insert-disc-demo"));
    }
    let real_launch = has("--real-launch") || (!cfg!(debug_assertions) && !has("--log-launch"));
    let mode = if has("--fake-drive") {
        DriveMode::Fake
    } else if has("--real-drive") || !cfg!(debug_assertions) {
        if cfg!(windows) { DriveMode::Windows } else { DriveMode::None }
    } else {
        DriveMode::Fake
    };
    Host::open(data_dir(), mode, real_launch)
}

/// Controle no Rust (D1): eventos de navegação para a UI, só com a janela em foco (SECURITY R10).
#[cfg(feature = "gilrs")]
fn start_gamepad(app: tauri::AppHandle) {
    use tauri::Emitter;
    std::thread::spawn(move || {
        let Some(mut source) = insert_disc_host::gamepad::GamepadSource::new() else { return };
        loop {
            std::thread::sleep(Duration::from_millis(8));
            let events = source.poll(); // sempre esvazia a fila, mesmo sem foco
            let focused = app.get_webview_window("main").and_then(|w| w.is_focused().ok()).unwrap_or(false);
            if !focused {
                continue;
            }
            for e in events {
                let _ = app.emit("pad", serde_json::json!({ "name": e.name, "pressed": e.pressed, "device": e.device }));
            }
        }
    });
}

fn main() {
    let host: Shared = Arc::new(Mutex::new(build_host()));
    let ticker = host.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(50));
        ticker.lock().unwrap().tick();
    });

    tauri::Builder::default()
        // instância única (Q22): abrir de novo traz a janela existente para a frente
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .setup(|_app| {
            #[cfg(feature = "gilrs")]
            start_gamepad(_app.handle().clone());
            Ok(())
        })
        .manage(host)
        // capas salvas: só nomes gerados pelo app (`Host::cover_file` valida), nunca caminhos
        .register_uri_scheme_protocol("cover", |ctx, request| {
            let host = ctx.app_handle().state::<Shared>();
            let name = request.uri().path().trim_start_matches('/').to_string();
            let bytes = host.lock().unwrap().cover_file(&name).and_then(|p| std::fs::read(p).ok());
            match bytes {
                Some(b) => tauri::http::Response::builder().header("Content-Type", "image/jpeg").body(b).unwrap(),
                None => tauri::http::Response::builder().status(404).body(Vec::new()).unwrap(),
            }
        })
        .invoke_handler(tauri::generate_handler![snapshot, intent, export_catalog, import_catalog, steam_games, set_cover, dev_state, dev])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o app");
}
