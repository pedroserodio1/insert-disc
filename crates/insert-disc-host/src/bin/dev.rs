//! Servidor de desenvolvimento: serve a UI e a API JSON sobre o núcleo com o drive falso,
//! para rodar tudo no navegador sem o Tauri. `cargo run -p insert-disc-host [-- porta]`

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use insert_disc_host::Host;
use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response, Server};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "png" => "image/png",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

fn json_response(v: Value, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(v.to_string()).with_status_code(status).with_header(Header::from_bytes("Content-Type", "application/json").unwrap())
}

fn demo_cover(n: usize) -> String {
    let hues = [210, 340, 8, 150, 40, 270, 190, 20];
    let h = hues[n % hues.len()];
    format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 600 900'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'>\
<stop offset='0' stop-color='hsl({h},55%,30%)'/><stop offset='1' stop-color='hsl({},60%,12%)'/></linearGradient></defs>\
<rect width='600' height='900' fill='url(#g)'/><circle cx='300' cy='360' r='170' fill='none' stroke='hsla({h},70%,75%,.55)' stroke-width='10'/>\
<circle cx='300' cy='360' r='90' fill='hsla({h},70%,75%,.25)'/><rect x='60' y='700' width='480' height='14' fill='hsla({h},70%,80%,.6)'/>\
<rect x='60' y='740' width='300' height='14' fill='hsla({h},70%,80%,.35)'/></svg>",
        (h + 40) % 360
    )
}

fn handle(host: &Arc<Mutex<Host>>, mut req: Request) {
    let url = req.url().split('?').next().unwrap_or("/").to_string();
    let method = req.method().clone();
    let mut body = String::new();
    let _ = req.as_reader().read_to_string(&mut body);
    let parsed: Value = serde_json::from_str(&body).unwrap_or(Value::Null);

    let resp = match (method, url.as_str()) {
        (Method::Get, "/api/snapshot") => {
            Response::from_string(host.lock().unwrap().snapshot_json()).with_header(Header::from_bytes("Content-Type", "application/json").unwrap())
        }
        (Method::Post, "/api/intent") => match host.lock().unwrap().intent(&parsed) {
            Ok(v) => json_response(v, 200),
            Err(e) => json_response(json!({ "error": e }), 400),
        },
        (Method::Get, "/api/export") => Response::from_string(host.lock().unwrap().export_json()).with_header(Header::from_bytes("Content-Type", "application/json").unwrap()),
        (Method::Post, "/api/import") => match host.lock().unwrap().import_json(&body) {
            Ok(()) => json_response(json!({ "ok": true }), 200),
            Err(e) => json_response(json!({ "error": e }), 400),
        },
        (Method::Get, "/api/dev") => json_response(host.lock().unwrap().dev_state(), 200),
        (Method::Get, "/api/steam") => json_response(host.lock().unwrap().steam_games(), 200),
        (Method::Post, "/api/dev") => match host.lock().unwrap().dev(&parsed) {
            Ok(v) => json_response(v, 200),
            Err(e) => json_response(json!({ "error": e }), 400),
        },
        (Method::Get, p) if p.starts_with("/demo-cover/") => {
            let n = p.trim_start_matches("/demo-cover/").trim_end_matches(".svg").parse().unwrap_or(0);
            Response::from_string(demo_cover(n)).with_header(Header::from_bytes("Content-Type", "image/svg+xml").unwrap())
        }
        (Method::Get, p) => {
            let rel = if p == "/" { "index.html" } else { p.trim_start_matches('/') };
            let path = ui_dir().join(rel);
            match path.canonicalize().ok().filter(|c| ui_dir().canonicalize().is_ok_and(|root| c.starts_with(root))).and_then(|c| std::fs::read(c).ok()) {
                Some(bytes) => Response::from_data(bytes).with_header(Header::from_bytes("Content-Type", content_type(rel)).unwrap()).with_header(Header::from_bytes("Cache-Control", "no-store").unwrap()),
                None => Response::from_string("não encontrado").with_status_code(404),
            }
        }
        _ => Response::from_string("método não permitido").with_status_code(405),
    };
    let _ = req.respond(resp);
}

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|p| p.parse().ok()).unwrap_or(5173);
    let host = Arc::new(Mutex::new(Host::demo(std::env::temp_dir().join("insert-disc-demo"))));

    let ticker = host.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(50));
        ticker.lock().unwrap().tick();
    });

    let server = Server::http(("127.0.0.1", port)).expect("porta ocupada");
    println!("Insert Disc (dev) em http://127.0.0.1:{port}/  (?dev=1 abre o painel do drive falso)");
    for req in server.incoming_requests() {
        handle(&host, req);
    }
}
