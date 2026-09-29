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
    let mut h = Host::open(&dir, true);
    let s = snap(&mut h);
    assert_eq!((s["state"].as_str(), s["library"].as_array().unwrap().len()), (Some("LIBRARY"), 0));
    assert_eq!(h.intent(&json!({ "type": "add_game" })).unwrap(), json!({ "ok": true }));
    h.app.drive_mut().insert_blank_cdrw();
    snap(&mut h); // processa a chegada do disco
    h.intent(&json!({ "type": "create_game", "name": "Portal", "kind": "steam", "app_id": 400 })).unwrap();
    assert!(dir.join("catalog.json").exists());
    drop(h);

    let mut h = Host::open(&dir, false); // "fechar e abrir" mantém o jogo
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
    let mut h = Host::open(&dir, false);
    assert_eq!(snap(&mut h)["state"], "CATALOG_ERROR");
    assert_eq!(std::fs::read_to_string(dir.join("catalog.json")).unwrap(), "{ não é json");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[should_panic(expected = "pasta de demonstração inválida")]
fn demo_refuses_to_wipe_a_folder_that_is_not_a_demo_folder() {
    let _ = Host::demo(std::env::temp_dir().join("InsertDisc"));
}
