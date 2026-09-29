//! Camada entre a UI e o núcleo (docs/UI-CONTRACT.md): traduz JSON em `Intent`, devolve o
//! `Snapshot` como JSON e oferece controles de desenvolvimento do drive falso.
//! Nada aqui depende de Tauri: o app desktop e o servidor de desenvolvimento usam o mesmo `Host`.

pub mod demo;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use insert_disc_core::app::*;
use insert_disc_core::catalog::*;
use insert_disc_core::drive::{Capabilities, Tri};
use insert_disc_core::fake::FakeIsoDrive;
use insert_disc_core::launch::{LaunchError, LaunchRequest, Launcher};
use serde_json::{json, Value};
use uuid::Uuid;

/// Lançador de desenvolvimento: só registra o que seria executado (nunca abre jogo).
#[derive(Clone, Default)]
pub struct LogLauncher {
    pub log: Arc<Mutex<Vec<String>>>,
    pub fail: Arc<Mutex<Option<LaunchError>>>,
}

impl Launcher for LogLauncher {
    fn launch(&mut self, req: &LaunchRequest) -> Result<(), LaunchError> {
        if let Some(e) = self.fail.lock().unwrap().clone() {
            return Err(e);
        }
        self.log.lock().unwrap().push(format!("{req:?}"));
        Ok(())
    }
}

pub struct Host {
    pub app: App<FakeIsoDrive, LogLauncher>,
    launcher: LogLauncher,
    start: Instant,
    demo: Option<demo::Demo>,
}

fn uuid_of(v: &Value, key: &str) -> Result<Uuid, String> {
    v.get(key).and_then(Value::as_str).and_then(|s| s.parse().ok()).ok_or_else(|| format!("campo inválido: {key}"))
}

fn str_of<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key).and_then(Value::as_str).ok_or_else(|| format!("campo inválido: {key}"))
}

fn parse_action(id: &str) -> Result<Action, String> {
    Ok(match id {
        "play_other" => Action::PlayOther,
        "try_other" => Action::TryOther,
        "adopt" => Action::Adopt,
        "burn" => Action::Burn,
        "confirm" => Action::Confirm,
        "continue" => Action::Continue,
        "done" => Action::Done,
        "retry_tray" => Action::RetryTray,
        "erase_retry" => Action::EraseRetry,
        "choose_drive" => Action::ChooseDrive,
        "burn_another" => Action::BurnAnother,
        "remove_game" => Action::RemoveGame,
        "start_empty" => Action::StartEmpty,
        s => {
            if let Some(d) = s.strip_prefix("unlink_disc:") {
                Action::UnlinkDisc(d.parse().map_err(|_| "disc inválido")?)
            } else if let Some(n) = s.strip_prefix("restore_backup:") {
                Action::RestoreBackup(n.parse().map_err(|_| "índice inválido")?)
            } else {
                return Err(format!("ação desconhecida: {s}"));
            }
        }
    })
}

fn parse_setting(v: &Value) -> Result<SettingChange, String> {
    let val = v.get("value");
    Ok(match str_of(v, "key")? {
        "drive" => SettingChange::Drive(val.and_then(Value::as_str).map(String::from)),
        "on_disc_insert" => SettingChange::OnDiscInsert(match val.and_then(Value::as_str) {
            Some("launch") => OnDiscInsert::Launch,
            Some("focus") => OnDiscInsert::Focus,
            _ => return Err("valor inválido".into()),
        }),
        "loading_min_ms" => SettingChange::LoadingMinMs(val.and_then(Value::as_u64).ok_or("valor inválido")?.min(60_000) as u32),
        "locale" => SettingChange::Locale(val.and_then(Value::as_str).map(String::from)),
        "window_mode" => SettingChange::WindowMode(match val.and_then(Value::as_str) {
            Some("fullscreen") => WindowMode::Fullscreen,
            Some("windowed") => WindowMode::Windowed,
            _ => return Err("valor inválido".into()),
        }),
        "covers_online" => SettingChange::CoversOnline(val.and_then(Value::as_bool).ok_or("valor inválido")?),
        k => return Err(format!("configuração desconhecida: {k}")),
    })
}

fn parse_intent(v: &Value) -> Result<Intent, String> {
    Ok(match str_of(v, "type")? {
        "select" => Intent::Select(uuid_of(v, "game_id")?),
        "back" => Intent::Back,
        "action" => Intent::Action(parse_action(str_of(v, "id")?)?),
        "hold_complete" => Intent::HoldComplete,
        "add_game" => Intent::AddGame,
        "options" => Intent::Options(uuid_of(v, "game_id")?),
        "settings" => Intent::OpenSettings,
        "choose_game" => Intent::ChooseGame(uuid_of(v, "game_id")?),
        "label_edit" => Intent::LabelEdit(str_of(v, "text")?.to_string()),
        "rename_game" => Intent::RenameGame(uuid_of(v, "game_id")?, str_of(v, "name")?.to_string()),
        "set_setting" => Intent::SetSetting(parse_setting(v)?),
        "create_game" => {
            let name = str_of(v, "name")?.to_string();
            let kind = match str_of(v, "kind")? {
                "steam" => GameKind::Steam { app_id: v.get("app_id").and_then(Value::as_u64).ok_or("app_id inválido")? as u32 },
                "custom" => GameKind::Custom {
                    executable: str_of(v, "executable")?.into(),
                    args: v.get("args").and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
                    working_dir: v.get("working_dir").and_then(Value::as_str).map(Into::into),
                    requires_elevation: v.get("requires_elevation").and_then(Value::as_bool).unwrap_or(false),
                },
                k => return Err(format!("tipo de jogo desconhecido: {k}")),
            };
            Intent::CreateGame(NewGame { name, kind })
        }
        t => return Err(format!("intenção desconhecida: {t}")),
    })
}

impl Host {
    pub fn demo(dir: impl Into<std::path::PathBuf>) -> Self {
        let dir = dir.into();
        let (catalog, demo) = demo::build(&dir);
        let launcher = LogLauncher::default();
        let mut drive = FakeIsoDrive::new(dir.join("burned"));
        demo.insert_default(&mut drive);
        Host { app: App::boot(Ok(catalog), None, drive, launcher.clone(), 0), launcher, start: Instant::now(), demo: Some(demo) }
    }

    /// Modo real: catálogo em `data_dir/catalog.json` (ausente = estante vazia; corrompido =
    /// `CATALOG_ERROR`, sem sobrescrever). Ainda não há drive físico (W2-W5): sem `fake_drive`
    /// o app fica "sem drive"; com ele, o drive falso serve para experimentar (Q7, SECURITY R8).
    pub fn open(data_dir: impl Into<std::path::PathBuf>, fake_drive: bool) -> Self {
        let dir = data_dir.into();
        let path = dir.join("catalog.json");
        let launcher = LogLauncher::default();
        let mut drive = FakeIsoDrive::new(dir.join("burned"));
        if !fake_drive {
            drive.disconnect();
        }
        let app = App::boot(Catalog::load(&path), Some(path), drive, launcher.clone(), 0);
        Host { app, launcher, start: Instant::now(), demo: None }
    }

    fn now(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn tick(&mut self) {
        let now = self.now();
        self.app.pump(now);
    }

    pub fn snapshot_json(&mut self) -> String {
        self.tick();
        serde_json::to_string(&self.app.snapshot()).expect("snapshot serializa")
    }

    /// Intenção inválida para o estado vira `{"ignored":true}` (a UI ignora, UI-CONTRACT).
    pub fn intent(&mut self, v: &Value) -> Result<Value, String> {
        let intent = parse_intent(v)?;
        let now = self.now();
        Ok(match self.app.dispatch(intent, now) {
            Ok(()) => json!({ "ok": true }),
            Err(IntentError::Ignored) => json!({ "ignored": true }),
            Err(IntentError::Invalid(e)) => json!({ "invalid": format!("{e:?}") }),
        })
    }

    pub fn export_json(&self) -> String {
        self.app.export_catalog()
    }

    pub fn import_json(&mut self, text: &str) -> Result<(), String> {
        self.app.import_catalog(text).map_err(|_| "estante inválida ou fora das configurações".to_string())
    }

    /// Estado do drive falso e do lançador, para o painel de desenvolvimento.
    pub fn dev_state(&mut self) -> Value {
        let caps = self.app.drive_mut().capabilities_now();
        let games: Vec<Value> = self
            .app
            .catalog
            .sorted_games()
            .iter()
            .map(|g| json!({ "game_id": g.game_id, "name": g.name, "has_disc": !g.discs.is_empty() }))
            .collect();
        json!({
            "has_media": self.app.drive().has_media(),
            "media_present": self.app.media_present(),
            "armed": self.app.armed(),
            "launched": *self.launcher.log.lock().unwrap(),
            "caps": {
                "tray_open": caps.tray_open == Tri::Yes, "eject": caps.eject == Tri::Yes,
                "write_cdr": caps.write_cdr == Tri::Yes, "write_cdrw": caps.write_cdrw == Tri::Yes,
            },
            "games": games,
            "scenarios": self.demo.as_ref().map(|d| d.scenario_names()).unwrap_or_default(),
        })
    }

    pub fn dev(&mut self, v: &Value) -> Result<Value, String> {
        let cmd = str_of(v, "cmd")?;
        match cmd {
            "insert" => {
                let what = str_of(v, "what")?.to_string();
                // um disco já no drive é trocado: remove e insere (eventos em sequência)
                if self.app.drive().has_media() {
                    self.app.drive_mut().remove_media();
                }
                self.demo.as_ref().ok_or("sem modo demonstração")?.insert(self.app.drive_mut(), &what)?;
            }
            "remove" => self.app.drive_mut().remove_media(),
            "duplicate" => self.app.drive_mut().inject_duplicate_arrival(),
            "disconnect" => self.app.drive_mut().disconnect(),
            "reconnect" => self.app.drive_mut().reconnect(),
            "fail_burn" => self.app.drive_mut().set_fail_burn(v["on"].as_bool().unwrap_or(true)),
            "fail_erase" => self.app.drive_mut().set_fail_erase(v["on"].as_bool().unwrap_or(true)),
            "fail_open_tray" => self.app.drive_mut().set_fail_open_tray(v["on"].as_bool().unwrap_or(true)),
            "cap" => {
                let d = self.app.drive_mut();
                let mut c: Capabilities = d.capabilities_now();
                let t = if v["on"].as_bool().unwrap_or(true) { Tri::Yes } else { Tri::No };
                match str_of(v, "name")? {
                    "tray_open" => c.tray_open = t,
                    "eject" => c.eject = t,
                    "write_cdr" => c.write_cdr = t,
                    "write_cdrw" => c.write_cdrw = t,
                    n => return Err(format!("capacidade desconhecida: {n}")),
                }
                d.set_capabilities(c);
            }
            "launcher_fail" => {
                *self.launcher.fail.lock().unwrap() = match v["reason"].as_str() {
                    Some("steam_missing") => Some(LaunchError::SteamMissing),
                    Some("not_found") => Some(LaunchError::NotFound),
                    Some("elevation_denied") => Some(LaunchError::ElevationDenied),
                    _ => None,
                };
            }
            c => return Err(format!("comando desconhecido: {c}")),
        }
        Ok(json!({ "ok": true }))
    }
}
