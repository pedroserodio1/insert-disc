use std::time::Duration;

use insert_disc_host::Host;
use serde_json::{json, Value};

fn snap(h: &mut Host) -> Value {
    serde_json::from_str(&h.snapshot_json()).unwrap()
}

#[test]
fn play_flow_through_the_json_api() {
    let dir = std::env::temp_dir().join(format!("insert-disc-test-{}", std::process::id()));
    let mut h = Host::demo(&dir);
    let s = snap(&mut h);
    assert_eq!(s["state"], "LIBRARY");
    assert_eq!(s["version"], 1);
    let lib = s["library"].as_array().unwrap();
    assert!(lib.len() >= 10 && lib.windows(2).all(|w| w[0]["name"].as_str().unwrap().to_lowercase() <= w[1]["name"].as_str().unwrap().to_lowercase()));
    let hk = lib.iter().find(|g| g["name"] == "Hollow Knight").unwrap()["game_id"].as_str().unwrap().to_string();

    assert_eq!(h.intent(&json!({ "type": "select", "game_id": hk })).unwrap(), json!({ "ok": true }));
    let s = snap(&mut h);
    assert_eq!((s["state"].as_str(), s["reason"].as_str()), (Some("WAITING_DISC"), Some("tray_opening")));
    assert_eq!(s["game"]["name"], "Hollow Knight");

    h.dev(&json!({ "cmd": "insert", "what": format!("game:{hk}") })).unwrap();
    assert_eq!(snap(&mut h)["state"], "IDENTIFIED");
    assert_eq!(snap(&mut h)["media"]["class"], "MATCH");
    std::thread::sleep(Duration::from_millis(700));
    let s = snap(&mut h);
    assert_eq!(s["state"], "LAUNCHING");
    assert_eq!(s["timing"]["min_ms"], 3000);
    assert!(h.dev_state()["launched"][0].as_str().unwrap().contains("steam://run/367520"));

    // intenção inválida para o estado: ignorada, não erro
    assert_eq!(h.intent(&json!({ "type": "add_game" })).unwrap(), json!({ "ignored": true }));
    // entrada malformada: erro
    assert!(h.intent(&json!({ "type": "select" })).is_err());
    assert!(h.intent(&json!({ "type": "nope" })).is_err());
    assert!(h.dev(&json!({ "cmd": "nope" })).is_err());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn registration_and_rejection_scenarios_are_reachable_from_the_dev_commands() {
    let dir = std::env::temp_dir().join(format!("insert-disc-test2-{}", std::process::id()));
    let mut h = Host::demo(&dir);
    h.intent(&json!({ "type": "add_game" })).unwrap();
    h.dev(&json!({ "cmd": "insert", "what": "blank_cdrw" })).unwrap();
    assert_eq!(snap(&mut h)["state"], "REG_CHOOSE_GAME");
    h.intent(&json!({ "type": "create_game", "name": "Novo Jogo", "kind": "steam", "app_id": 1 })).unwrap();
    let s = snap(&mut h);
    assert_eq!(s["state"], "REG_LABEL_PREVIEW");
    assert_eq!(s["actions"][0]["id"], "continue");
    h.intent(&json!({ "type": "action", "id": "continue" })).unwrap();
    assert_eq!(snap(&mut h)["state"], "BURN_DONE");
    let bad = h.intent(&json!({ "type": "create_game", "name": "x", "kind": "steam", "app_id": 0 })).unwrap();
    assert_eq!(bad, json!({ "ignored": true })); // já saiu de REG_CHOOSE_GAME
    let _ = std::fs::remove_dir_all(dir);
}

fn temp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("insert-disc-{name}-{}", std::process::id()))
}

#[test]
fn real_mode_persists_the_shelf_and_starts_without_a_drive() {
    let dir = temp("open");
    let _ = std::fs::remove_dir_all(&dir);
    let mut h = Host::open(&dir, true, false);
    let s = snap(&mut h);
    assert_eq!((s["state"].as_str(), s["library"].as_array().unwrap().len()), (Some("LIBRARY"), 0));
    assert_eq!(h.intent(&json!({ "type": "add_game" })).unwrap(), json!({ "ok": true }));
    h.app.drive_mut().insert_blank_cdrw();
    snap(&mut h); // processa a chegada do disco
    h.intent(&json!({ "type": "create_game", "name": "Portal", "kind": "steam", "app_id": 400 })).unwrap();
    assert!(dir.join("catalog.json").exists());
    drop(h);

    let mut h = Host::open(&dir, false, false); // "fechar e abrir" mantém o jogo
    assert!(snap(&mut h)["library"].as_array().unwrap().iter().any(|g| g["name"] == "Portal"));
    assert_eq!(h.dev_state()["scenarios"], json!([]));
    assert!(h.dev(&json!({ "cmd": "insert", "what": "unknown" })).is_err()); // sem modo demonstração
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn corrupt_catalog_is_reported_and_left_untouched() {
    let dir = temp("corrupt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("catalog.json"), "{ não é json").unwrap();
    let mut h = Host::open(&dir, false, false);
    assert_eq!(snap(&mut h)["state"], "CATALOG_ERROR");
    assert_eq!(std::fs::read_to_string(dir.join("catalog.json")).unwrap(), "{ não é json");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[should_panic(expected = "pasta de demonstração inválida")]
fn demo_refuses_to_wipe_a_folder_that_is_not_a_demo_folder() {
    let _ = Host::demo(std::env::temp_dir().join("InsertDisc"));
}

#[test]
fn slow_drive_is_visible_through_the_json_api_and_the_ui_keeps_responding() {
    let dir = temp("slow");
    let mut h = Host::demo(std::env::temp_dir().join(format!("insert-disc-slow-{}", std::process::id())));
    h.dev(&json!({ "cmd": "op_delay", "ms": 400 })).unwrap();
    assert_eq!(h.dev_state()["op_delay_ms"], 400);
    h.intent(&json!({ "type": "add_game" })).unwrap();
    h.dev(&json!({ "cmd": "insert", "what": "blank_cdrw" })).unwrap();
    assert_eq!(snap(&mut h)["state"], "REG_READING");
    assert!(h.intent(&json!({ "type": "options", "game_id": uuid_zero() })).unwrap().get("ignored").is_some()); // responde durante a leitura
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(snap(&mut h)["state"], "REG_CHOOSE_GAME");
    let _ = std::fs::remove_dir_all(dir);
}

fn uuid_zero() -> String {
    "00000000-0000-0000-0000-000000000000".into()
}

fn png_b64(w: u32, h: u32, px: [u8; 3]) -> String {
    use base64::Engine;
    let img: image::RgbImage = image::ImageBuffer::from_pixel(w, h, image::Rgb(px));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    base64::engine::general_purpose::STANDARD.encode(out.into_inner())
}

#[test]
fn cover_import_saves_a_clean_jpeg_sets_the_spine_and_replaces_the_old_file() {
    use base64::Engine;
    let mut h = Host::demo(std::env::temp_dir().join(format!("insert-disc-covers-{}", std::process::id())));
    let s = snap(&mut h);
    let celeste = s["library"].as_array().unwrap().iter().find(|g| g["name"] == "Celeste").unwrap()["game_id"].as_str().unwrap().to_string();

    h.set_cover_json(&json!({ "game_id": celeste, "data": png_b64(800, 1200, [240, 240, 60]) })).unwrap();
    let s = snap(&mut h);
    let g = s["library"].as_array().unwrap().iter().find(|g| g["game_id"] == json!(celeste)).unwrap().clone();
    let name = g["cover"].as_str().unwrap().strip_prefix("cover:").unwrap().to_string();
    assert!(name.ends_with(".jpg"));
    let spine = g["spine_color"].as_str().unwrap();
    assert!(spine.starts_with('#') && spine.len() == 7);
    let file = h.cover_file(&name).unwrap();
    assert_eq!(&std::fs::read(&file).unwrap()[..2], &[0xFF, 0xD8]); // JPEG reencodado

    h.set_cover_json(&json!({ "game_id": celeste, "data": png_b64(64, 96, [10, 10, 200]) })).unwrap();
    assert!(!file.exists(), "capa antiga apagada");

    // recusas: SVG, lixo, base64 inválido, jogo inexistente
    let svg = base64::engine::general_purpose::STANDARD.encode("<svg xmlns='http://www.w3.org/2000/svg'/>");
    for data in [svg, base64::engine::general_purpose::STANDARD.encode("lixo"), "@@@".into()] {
        assert!(h.set_cover_json(&json!({ "game_id": celeste, "data": data })).is_err());
    }
    assert!(h.set_cover_json(&json!({ "game_id": uuid_zero(), "data": png_b64(8, 8, [1, 2, 3]) })).is_err());
    assert!(h.cover_file("../catalog.json").is_none());
}
