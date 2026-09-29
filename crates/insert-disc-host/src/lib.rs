//! Camada entre a UI e o núcleo (docs/UI-CONTRACT.md): traduz JSON em `Intent`, devolve o
//! `Snapshot` como JSON e oferece controles de desenvolvimento do drive falso.
//! Nada aqui depende de Tauri: o app desktop e o servidor de desenvolvimento usam o mesmo `Host`.

pub mod anydrive;
pub mod covers;
pub mod demo;
#[cfg(feature = "gilrs")]
pub mod gamepad;
pub mod launcher;
pub mod log;
pub mod steam;
pub mod windrive;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use insert_disc_core::app::*;
use insert_disc_core::catalog::*;
use insert_disc_core::drive::{Capabilities, Tri};
use insert_disc_core::fake::FakeIsoDrive;

pub use anydrive::AnyDrive;

/// Qual drive o app usa (Q7): o do Windows, o falso (ISO) ou nenhum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveMode {
    Windows,
    Fake,
    None,
}

const NO_FAKE: &str = "o drive falso não está em uso";
use insert_disc_core::launch::{LaunchError, LaunchRequest, Launcher};
use serde_json::{json, Value};
use uuid::Uuid;

/// Lançador de desenvolvimento: registra o que seria executado e, só com `real`, executa de verdade
/// (`launcher::launch_real`). O padrão nunca abre jogo.
#[derive(Clone, Default)]
pub struct LogLauncher {
    pub log: Arc<Mutex<Vec<String>>>,
    pub fail: Arc<Mutex<Option<LaunchError>>>,
    pub real: bool,
}

impl Launcher for LogLauncher {
    fn launch(&mut self, req: &LaunchRequest) -> Result<(), LaunchError> {
        if let Some(e) = self.fail.lock().unwrap().clone() {
            return Err(e);
        }
        self.log.lock().unwrap().push(format!("{req:?}"));
        if self.real { launcher::launch_real(req) } else { Ok(()) }
    }
}

pub struct Host {
    pub app: App<AnyDrive, LogLauncher>,
    launcher: LogLauncher,
    start: Instant,
    demo: Option<demo::Demo>,
    /// Pasta das capas salvas.
    covers_dir: Option<std::path::PathBuf>,
    logger: Option<log::Logger>,
    last_state: &'static str,
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

fn parse_new_game(v: &Value) -> Result<NewGame, String> {
    let name = str_of(v, "name")?.to_string();
    let kind = match str_of(v, "kind")? {
        "steam" => GameKind::Steam { app_id: v.get("app_id").and_then(Value::as_u64).and_then(|n| u32::try_from(n).ok()).ok_or("app_id inválido")? },
        "custom" => GameKind::Custom {
            executable: str_of(v, "executable")?.into(),
            args: v.get("args").and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
            working_dir: v.get("working_dir").and_then(Value::as_str).map(Into::into),
            requires_elevation: v.get("requires_elevation").and_then(Value::as_bool).unwrap_or(false),
        },
        k => return Err(format!("tipo de jogo desconhecido: {k}")),
    };
    Ok(NewGame { name, kind })
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
        "create_game" => Intent::CreateGame(parse_new_game(v)?),
        "update_game" => Intent::UpdateGame(uuid_of(v, "game_id")?, parse_new_game(v)?),
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
        Host { app: App::boot(Ok(catalog), None, AnyDrive::Fake(drive), launcher.clone(), 0), launcher, start: Instant::now(), demo: Some(demo), covers_dir: Some(dir.join("covers")), logger: None, last_state: "" }
    }

    /// Modo real: catálogo em `data_dir/catalog.json` (ausente = estante vazia; corrompido =
    /// `CATALOG_ERROR`, sem sobrescrever). O drive vem de `mode` (Q7, SECURITY R8).
    pub fn open(data_dir: impl Into<std::path::PathBuf>, mode: DriveMode, real_launch: bool) -> Self {
        let dir = data_dir.into();
        let path = dir.join("catalog.json");
        let launcher = LogLauncher { real: real_launch, ..Default::default() };
        let drive = match mode {
            DriveMode::Windows => AnyDrive::Windows(windrive::WindowsDrive::new()),
            DriveMode::Fake => AnyDrive::Fake(FakeIsoDrive::new(dir.join("burned"))),
            DriveMode::None => {
                let mut d = FakeIsoDrive::new(dir.join("burned"));
                d.disconnect(); // sem unidade: o app mostra "sem drive"
                AnyDrive::Fake(d)
            }
        };
        let app = App::boot(Catalog::load(&path), Some(path), drive, launcher.clone(), 0);
        let mut logger = log::Logger::open(dir.join("logs").join("insert-disc.log"), std::env::var("INSERT_DISC_LOG").is_ok_and(|v| v == "debug"));
        logger.info(&format!("início: versão {} drive={mode:?} lançador_real={real_launch}", env!("CARGO_PKG_VERSION")));
        Host { app, launcher, start: Instant::now(), demo: None, covers_dir: Some(dir.join("covers")), logger: Some(logger), last_state: "" }
    }

    fn now(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn tick(&mut self) {
        let now = self.now();
        self.app.pump(now);
        self.log_state_change();
    }

    /// Uma linha por mudança de estado (nomes e classes, nunca caminhos); a requisição de
    /// lançamento, que tem caminho de executável, só em depuração.
    fn log_state_change(&mut self) {
        let name = self.app.state().name();
        if name == self.last_state {
            return;
        }
        self.last_state = name;
        let Some(l) = &mut self.logger else { return };
        match self.app.state() {
            State::LaunchError { reason, .. } => l.info(&format!("estado {name}: {}", reason.key())),
            State::Launching { .. } => {
                l.info(&format!("estado {name}"));
                if let Some(req) = self.launcher.log.lock().unwrap().last() {
                    l.debug(&format!("lançamento: {req}"));
                }
            }
            _ => l.info(&format!("estado {name}")),
        }
        if let Some(e) = &self.app.persist_error {
            l.info(&format!("erro ao salvar o catálogo: {e}"));
        }
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
            Ok(()) => {
                if v.get("steam_cover").and_then(Value::as_bool) == Some(true) {
                    self.steam_cover_for_last_game(); // capa é opcional: sem ela fica o placeholder
                }
                json!({ "ok": true })
            }
            Err(IntentError::Ignored) => json!({ "ignored": true }),
            Err(IntentError::Invalid(e)) => json!({ "invalid": format!("{e:?}") }),
        })
    }

    /// Capa de um jogo a partir de bytes de imagem (B1): valida, reencoda, salva em `covers/` e
    /// preenche `cover` e `spine_color`. Substitui (e apaga) a capa salva anterior.
    pub fn set_cover(&mut self, game_id: Uuid, bytes: &[u8], source: CoverSource) -> Result<(), String> {
        let dir = self.covers_dir.clone().ok_or("sem pasta de capas (modo demonstração)")?;
        self.app.catalog.game(game_id).ok_or("jogo inexistente")?;
        let cover = covers::process(bytes).map_err(|e| format!("capa recusada: {e:?}"))?;
        let name = format!("{}.jpg", Uuid::new_v4());
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(&name), &cover.jpeg).map_err(|e| e.to_string())?;
        let old = self.app.update_catalog(|c| {
            let g = c.games.iter_mut().find(|g| g.game_id == game_id)?;
            let old = g.cover.replace(format!("cover:{name}"));
            g.cover_source = Some(source);
            g.spine_color = Some(cover.spine);
            old
        });
        if let Some(old) = old.as_deref().and_then(|o| o.strip_prefix("cover:")).and_then(|o| covers::file_in(&dir, o)) {
            let _ = std::fs::remove_file(old);
        }
        Ok(())
    }

    /// Entrada da UI: `{game_id, data}` com `data` em base64 (imagem escolhida pelo usuário).
    pub fn set_cover_json(&mut self, v: &Value) -> Result<Value, String> {
        use base64::Engine;
        let id = uuid_of(v, "game_id")?;
        let b64 = str_of(v, "data")?;
        if b64.len() > covers::MAX_BYTES * 4 / 3 + 8 {
            return Err("capa recusada: TooBig".into());
        }
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).map_err(|_| "base64 inválido")?;
        self.set_cover(id, &bytes, CoverSource::UserFile)?;
        Ok(json!({ "ok": true }))
    }

    /// B2: depois de `create_game` de um jogo da Steam, copia a capa do cache da Steam pelo pipeline.
    fn steam_cover_for_last_game(&mut self) {
        let Some(g) = self.app.catalog.games.last() else { return };
        let GameKind::Steam { app_id } = g.kind else { return };
        let id = g.game_id;
        let Some(bytes) = steam::find_steam().and_then(|root| steam::cover_path(&root, app_id)).and_then(|p| std::fs::read(p).ok()) else { return };
        let _ = self.set_cover(id, &bytes, CoverSource::SteamCache);
    }

    /// Arquivo de uma capa salva, para o protocolo de capas (só nomes gerados pelo app).
    pub fn cover_file(&self, name: &str) -> Option<std::path::PathBuf> {
        covers::file_in(self.covers_dir.as_deref()?, name)
    }

    pub fn export_json(&self) -> String {
        self.app.export_catalog()
    }

    pub fn import_json(&mut self, text: &str) -> Result<(), String> {
        self.app.import_catalog(text).map_err(|_| "estante inválida ou fora das configurações".to_string())
    }

    /// Jogos instalados na Steam deste PC (A3): `{available, games:[{app_id,name}]}`.
    pub fn steam_games(&self) -> Value {
        let root = steam::find_steam();
        let games: Vec<Value> = root.as_deref().map(steam::installed_games).unwrap_or_default().into_iter().map(|g| json!({ "app_id": g.app_id, "name": g.name })).collect();
        json!({ "available": root.is_some(), "games": games })
    }

    /// Estado do drive falso e do lançador, para o painel de desenvolvimento.
    pub fn dev_state(&mut self) -> Value {
        let fake = self.app.drive().fake();
        let caps = fake.map_or(Capabilities::ALL, |d| d.capabilities_now());
        let games: Vec<Value> = self
            .app
            .catalog
            .sorted_games()
            .iter()
            .map(|g| json!({ "game_id": g.game_id, "name": g.name, "has_disc": !g.discs.is_empty() }))
            .collect();
        json!({
            "fake_drive": fake.is_some(),
            "has_media": fake.is_some_and(|d| d.has_media()),
            "media_present": self.app.media_present(),
            "armed": self.app.armed(),
            "op_delay_ms": fake.map_or(0, |d| d.op_delay_ms()),
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
                if self.app.drive().fake().is_some_and(|d| d.has_media()) {
                    self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.remove_media();
                }
                let fake = self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?;
                // mídias que não dependem das ISOs da demonstração valem em qualquer modo com drive falso
                match (what.as_str(), &self.demo) {
                    ("blank_cdr", _) => fake.insert_blank_cdr(),
                    ("blank_cdrw", _) => fake.insert_blank_cdrw(),
                    ("audio", _) => fake.insert_audio(),
                    ("unreadable", _) => fake.insert_unreadable(),
                    (_, Some(demo)) => demo.insert(fake, &what)?,
                    _ => return Err("esse cenário só existe no modo demonstração (--demo)".into()),
                }
            }
            "remove" => self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.remove_media(),
            "op_delay" => self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.set_op_delay_ms(v["ms"].as_u64().unwrap_or(0).min(60_000)),
            "duplicate" => self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.inject_duplicate_arrival(),
            "disconnect" => self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.disconnect(),
            "reconnect" => self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.reconnect(),
            "fail_burn" => self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.set_fail_burn(v["on"].as_bool().unwrap_or(true)),
            "fail_erase" => self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.set_fail_erase(v["on"].as_bool().unwrap_or(true)),
            "fail_open_tray" => self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?.set_fail_open_tray(v["on"].as_bool().unwrap_or(true)),
            "cap" => {
                let d = self.app.drive_mut().fake_mut().ok_or(NO_FAKE)?;
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
