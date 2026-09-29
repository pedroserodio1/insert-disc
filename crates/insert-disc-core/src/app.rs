//! Máquina de estados + orquestração (docs/UX-STATES.md, docs/UI-CONTRACT.md).
//!
//! Tudo é síncrono e determinístico: o tempo entra como `now` (ms) em `dispatch`/`pump`.
//! Leitura, gravação e apagamento são operações assíncronas do drive (`start_op`/`poll_ops`): o
//! `App` guarda o contexto em `Pending` e continua quando `pump` recebe o resultado. O drive falso
//! sem atraso conclui na mesma chamada, o que mantém os testes determinísticos.

use std::path::PathBuf;

use uuid::Uuid;

use crate::catalog::*;
use crate::classify::{classify, MediaClass};
use crate::drive::*;
use crate::gameini;
use crate::launch::{request_for, LaunchError, Launcher};

/// Tempo mínimo da ficha em `MATCH` antes de lançar (FRONTEND-DESIGN `dur-ficha-min`).
pub const FICHA_MIN_MS: u64 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayHint {
    Opening,
    Manual,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemReason {
    NoDrive,
    Removed,
    CannotBurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnFailure {
    WriteError,
    VerifyMismatch,
    DriveRemoved,
    EraseError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegRejectReason {
    CdrUsed,
    NotWritable,
    Audio,
    ReadError,
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Boot,
    CatalogError { backups: usize },
    Library,
    NoDiscYet { game: GameId },
    WaitingDisc { game: GameId, tray: TrayHint },
    Reading { game: GameId },
    Identified { game: GameId },
    Rejected { game: GameId, class: MediaClass },
    AdoptConfirm { game: GameId, disc: DiscId, ini_name: Option<String>, class: MediaClass },
    Launching { game: GameId, started: u64, min_ms: u32 },
    LaunchError { game: GameId, reason: LaunchError },
    GameOptions { game: GameId },
    RemoveConfirm { game: GameId },
    Settings,
    DriveProblem { reason: ProblemReason },
    RegInsert { game: Option<GameId>, tray: TrayHint },
    RegEraseConfirm { game: Option<GameId>, step: u8, old_disc: Option<DiscId> },
    Erasing { game: Option<GameId>, label: Option<String>, old_disc: Option<DiscId> },
    RegChooseGame,
    RegLabelPreview { game: GameId, label: String },
    RegCdrWarning { game: GameId, label: String },
    Burning { game: GameId, label: String },
    Verifying { game: GameId },
    BurnDone { game: GameId },
    BurnFailed { game: Option<GameId>, label: Option<String>, reason: BurnFailure, cdrw: bool },
    RegRejected { game: Option<GameId>, reason: RegRejectReason },
    /// Lendo o disco que chegou durante o cadastro (a leitura é assíncrona, A5).
    RegReading { game: Option<GameId> },
}

/// O que fazer com o resultado de uma leitura quando ela terminar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadThen {
    AutoInsert,
    Identify(GameId),
    Reg(Option<GameId>),
}

#[derive(Debug, Clone)]
struct BurnCtx {
    game: GameId,
    label: String,
    disc_id: DiscId,
    cdrw: bool,
    physical: Physical,
}

/// Operação do drive em andamento e o contexto para continuar quando terminar.
#[derive(Debug, Clone)]
enum Pending {
    Read(ReadThen),
    Erase { game: Option<GameId>, label: Option<String>, old: Option<DiscId> },
    Burn(BurnCtx),
    /// Releitura do disco gravado para conferir o `disc_id`.
    Verify(BurnCtx),
}

impl State {
    /// Nome estável, igual ao de UX-STATES.
    pub fn name(&self) -> &'static str {
        match self {
            State::Boot => "BOOT",
            State::CatalogError { .. } => "CATALOG_ERROR",
            State::Library => "LIBRARY",
            State::NoDiscYet { .. } => "NO_DISC_YET",
            State::WaitingDisc { .. } => "WAITING_DISC",
            State::Reading { .. } => "READING",
            State::Identified { .. } => "IDENTIFIED",
            State::Rejected { .. } => "REJECTED",
            State::AdoptConfirm { .. } => "ADOPT_CONFIRM",
            State::Launching { .. } => "LAUNCHING",
            State::LaunchError { .. } => "LAUNCH_ERROR",
            State::GameOptions { .. } => "GAME_OPTIONS",
            State::RemoveConfirm { .. } => "REMOVE_CONFIRM",
            State::Settings => "SETTINGS",
            State::DriveProblem { .. } => "DRIVE_PROBLEM",
            State::RegInsert { .. } => "REG_INSERT",
            State::RegEraseConfirm { .. } => "REG_ERASE_CONFIRM",
            State::Erasing { .. } => "ERASING",
            State::RegChooseGame => "REG_CHOOSE_GAME",
            State::RegLabelPreview { .. } => "REG_LABEL_PREVIEW",
            State::RegCdrWarning { .. } => "REG_CDR_WARNING",
            State::Burning { .. } => "BURNING",
            State::Verifying { .. } => "VERIFYING",
            State::BurnDone { .. } => "BURN_DONE",
            State::BurnFailed { .. } => "BURN_FAILED",
            State::RegRejected { .. } => "REG_REJECTED",
            State::RegReading { .. } => "REG_READING",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    PlayOther,
    TryOther,
    Adopt,
    Burn,
    Confirm,
    Continue,
    Done,
    RetryTray,
    EraseRetry,
    ChooseDrive,
    BurnAnother,
    RemoveGame,
    UnlinkDisc(DiscId),
    StartEmpty,
    RestoreBackup(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewGame {
    pub name: String,
    pub kind: GameKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingChange {
    Drive(Option<String>),
    OnDiscInsert(OnDiscInsert),
    LoadingMinMs(u32),
    Locale(Option<String>),
    WindowMode(WindowMode),
    CoversOnline(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    Select(GameId),
    Back,
    Action(Action),
    /// Segundo passo de `REG_ERASE_CONFIRM`.
    HoldComplete,
    AddGame,
    Options(GameId),
    OpenSettings,
    ChooseGame(GameId),
    CreateGame(NewGame),
    LabelEdit(String),
    RenameGame(GameId, String),
    /// Edição completa (A4): nome e tipo, com a validação da criação.
    UpdateGame(GameId, NewGame),
    SetSetting(SettingChange),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntentError {
    /// Inválida para o estado atual: a UI deve ignorar.
    Ignored,
    Invalid(CatalogError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Toast {
    Focus { game: GameId },
    Unknown,
    NotAGame,
    ReadError,
    DriveRemoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveStatus {
    Ok,
    None,
    Removed,
}

pub struct App<D: DriveBackend, L: Launcher> {
    pub catalog: Catalog,
    pub(crate) catalog_path: Option<PathBuf>,
    pub(crate) drive: D,
    pub(crate) launcher: L,
    pub(crate) drive_id: Option<String>,
    pub(crate) drive_status: DriveStatus,
    pub(crate) state: State,
    media_present: bool,
    armed: bool,
    pub(crate) last_media: Option<MediaInfo>,
    now: u64,
    ficha_since: u64,
    pub(crate) focus_hint: Option<GameId>,
    pub(crate) toast: Option<(u64, Toast)>,
    toast_seq: u64,
    pub(crate) progress: Option<u8>,
    pending: Option<Pending>,
    pub persist_error: Option<String>,
}

impl<D: DriveBackend, L: Launcher> App<D, L> {
    pub fn boot(load: Result<Catalog, LoadError>, path: Option<PathBuf>, drive: D, launcher: L, now: u64) -> Self {
        let (catalog, state) = match load {
            Ok(c) => (c, State::Library),
            Err(_) => {
                let n = path.as_deref().map(list_backups).map_or(0, |b| b.len());
                (Catalog::default(), State::CatalogError { backups: n })
            }
        };
        let mut app = App {
            catalog,
            catalog_path: path,
            drive,
            launcher,
            drive_id: None,
            drive_status: DriveStatus::None,
            state,
            media_present: false,
            armed: false,
            last_media: None,
            now,
            ficha_since: 0,
            focus_hint: None,
            toast: None,
            toast_seq: 0,
            progress: None,
            pending: None,
            persist_error: None,
        };
        app.select_drive();
        // Mídia já presente no boot fica desarmada; eventos antigos são descartados (ADR-0003).
        let _ = app.drive.poll_events();
        if let Some(d) = app.drive_id.clone() {
            app.media_present = app.drive.media_present(&d);
        }
        app
    }

    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn drive(&self) -> &D {
        &self.drive
    }
    pub fn drive_mut(&mut self) -> &mut D {
        &mut self.drive
    }
    pub fn launcher_mut(&mut self) -> &mut L {
        &mut self.launcher
    }
    pub fn launcher(&self) -> &L {
        &self.launcher
    }
    pub fn media_present(&self) -> bool {
        self.media_present
    }
    pub fn armed(&self) -> bool {
        self.armed
    }
    pub fn last_media(&self) -> Option<&MediaInfo> {
        self.last_media.as_ref()
    }

    /// Altera o catálogo por fora da máquina de estados (ex.: capa importada) e persiste.
    pub fn update_catalog<R>(&mut self, f: impl FnOnce(&mut Catalog) -> R) -> R {
        let r = f(&mut self.catalog);
        self.persist();
        r
    }

    /// Exporta a estante (JSON, sem segredos).
    pub fn export_catalog(&self) -> String {
        self.catalog.to_json()
    }

    /// Importar estante (Q19, opção a): substitui a atual. Só nas configurações.
    pub fn import_catalog(&mut self, json: &str) -> Result<(), LoadError> {
        if self.state != State::Settings {
            return Err(LoadError::Corrupt("fora das configurações".into()));
        }
        self.catalog = Catalog::from_json(json)?;
        self.persist();
        self.select_drive();
        Ok(())
    }

    // ---------- drive ----------

    fn select_drive(&mut self) {
        let list = self.drive.list_drives();
        let wanted = self.catalog.settings.drive.clone();
        let pick = list.iter().find(|d| Some(&d.id) == wanted.as_ref()).or(list.first()).map(|d| d.id.clone());
        self.drive_status = if pick.is_some() { DriveStatus::Ok } else if self.drive_id.is_some() { DriveStatus::Removed } else { DriveStatus::None };
        if pick.is_some() {
            self.drive_id = pick;
        }
    }

    pub(crate) fn caps(&mut self) -> Capabilities {
        match self.drive_id.clone() {
            Some(d) => self.drive.capabilities(&d),
            None => Capabilities { tray_open: Tri::No, tray_close: Tri::No, eject: Tri::No, write_cdr: Tri::No, write_cdrw: Tri::No, erase: Tri::No, media_events: Tri::No },
        }
    }

    fn drive_problem(&self) -> Option<ProblemReason> {
        match self.drive_status {
            DriveStatus::Ok => None,
            DriveStatus::None => Some(ProblemReason::NoDrive),
            DriveStatus::Removed => Some(ProblemReason::Removed),
        }
    }

    fn open_tray(&mut self) -> TrayHint {
        let Some(d) = self.drive_id.clone() else { return TrayHint::Manual };
        if self.drive.capabilities(&d).tray_open != Tri::Yes {
            return TrayHint::Manual;
        }
        match self.drive.open_tray(&d) {
            Ok(()) => TrayHint::Opening,
            Err(_) => TrayHint::Failed,
        }
    }

    fn eject(&mut self) {
        if let Some(d) = self.drive_id.clone() {
            let _ = self.drive.eject(&d); // sem suporte: o usuário tira à mão
        }
    }

    fn tray_after_removal(&mut self) -> TrayHint {
        if self.caps().tray_open == Tri::Yes { TrayHint::Opening } else { TrayHint::Manual }
    }

    // ---------- operações assíncronas do drive (A5) ----------

    /// Inicia a operação e, se o backend já a concluiu (drive falso sem atraso), continua na hora.
    fn begin(&mut self, op: DriveOp, pending: Pending) {
        match self.drive_id.clone() {
            Some(d) => {
                self.pending = Some(pending);
                self.drive.start_op(&d, op, self.now);
                self.drain_ops();
            }
            None if matches!(pending, Pending::Read(_)) => {
                self.pending = Some(pending);
                self.complete(OpResult::Read(Err(DriveError::Removed)));
            }
            None => {}
        }
    }

    fn drain_ops(&mut self) {
        loop {
            let mut finished = false;
            for u in self.drive.poll_ops(self.now) {
                match u {
                    OpUpdate::Progress(p) => {
                        if matches!(self.pending, Some(Pending::Burn(_) | Pending::Erase { .. })) {
                            self.progress = Some(p);
                        }
                    }
                    OpUpdate::Done(r) => {
                        finished = true;
                        self.complete(r);
                    }
                }
            }
            if !finished {
                return; // continuar só se o fim de uma operação pôde ter iniciado outra
            }
        }
    }

    fn complete(&mut self, r: OpResult) {
        let Some(p) = self.pending.take() else { return };
        match (p, r) {
            (Pending::Read(then), OpResult::Read(res)) => {
                let m = res.unwrap_or_else(|_| MediaInfo::unreadable());
                self.last_media = Some(m.clone());
                match then {
                    ReadThen::AutoInsert => self.after_auto_read(&m),
                    ReadThen::Identify(g) => self.after_identify(g, &m),
                    ReadThen::Reg(g) => self.after_reg_read(g, &m),
                }
            }
            (Pending::Erase { game, label, old }, OpResult::Erase(res)) => self.after_erase(game, label, old, res),
            (Pending::Burn(b), OpResult::Burn(res)) => self.after_burn(b, res),
            (Pending::Verify(b), OpResult::Read(res)) => self.after_verify(b, res),
            _ => {}
        }
    }

    fn persist(&mut self) {
        if let Some(p) = self.catalog_path.clone() {
            self.persist_error = self.catalog.save(&p).err().map(|e| e.to_string());
        }
    }

    fn set_toast(&mut self, t: Toast) {
        self.toast_seq += 1;
        self.toast = Some((self.toast_seq, t));
    }

    // ---------- tempo e eventos ----------

    pub fn pump(&mut self, now: u64) {
        self.now = now;
        for ev in self.drive.poll_events() {
            self.on_event(ev);
        }
        if self.pending.is_some() {
            self.drain_ops();
        }
        match self.state.clone() {
            State::Identified { game } if now.saturating_sub(self.ficha_since) >= FICHA_MIN_MS => self.start_launch(game),
            State::Launching { game, started, min_ms } if now.saturating_sub(started) >= min_ms as u64 => {
                self.focus_hint = Some(game);
                self.state = State::Library; // LAUNCHED é transitório
            }
            _ => {}
        }
    }

    fn on_event(&mut self, ev: DriveEvent) {
        match ev {
            DriveEvent::MediaArrived => {
                if self.media_present || self.drive_status != DriveStatus::Ok {
                    return; // deduplicação (UX-STATES)
                }
                self.media_present = true;
                self.armed = matches!(self.state, State::Library);
                match self.state.clone() {
                    State::Library => self.on_auto_insert(),
                    State::WaitingDisc { game, .. } => self.identify(game),
                    State::RegInsert { game, .. } => self.reg_read(game),
                    _ => {}
                }
            }
            DriveEvent::MediaRemoved => {
                self.media_present = false;
                self.armed = false;
                if matches!(self.pending, Some(Pending::Read(_))) {
                    self.pending = None; // leitura de um disco que já saiu
                }
                let tray = self.tray_after_removal();
                self.state = match self.state.clone() {
                    State::Reading { game } | State::Identified { game } | State::Rejected { game, .. } | State::AdoptConfirm { game, .. } => State::WaitingDisc { game, tray },
                    State::RegReading { game } | State::RegRejected { game, .. } | State::RegEraseConfirm { game, .. } => State::RegInsert { game, tray },
                    s => s,
                };
            }
            DriveEvent::DriveAdded => {
                self.select_drive();
                if matches!(self.state, State::DriveProblem { reason: ProblemReason::NoDrive | ProblemReason::Removed }) && self.drive_status == DriveStatus::Ok {
                    self.state = State::Library;
                }
            }
            DriveEvent::DriveRemoved => {
                self.select_drive();
                self.media_present = false;
                self.armed = false;
                self.pending = None;
                self.on_drive_removed();
            }
        }
    }

    /// Tabela "Drive removido em qualquer estado" de UX-STATES.
    fn on_drive_removed(&mut self) {
        use State::*;
        self.state = match self.state.clone() {
            Library | GameOptions { .. } | Settings | RemoveConfirm { .. } => {
                self.set_toast(Toast::DriveRemoved);
                self.state.clone()
            }
            NoDiscYet { .. } | WaitingDisc { .. } | Reading { .. } | Identified { .. } | Rejected { .. } | AdoptConfirm { .. } | RegInsert { .. } | RegReading { .. }
            | RegEraseConfirm { .. } | RegChooseGame | RegLabelPreview { .. } | RegCdrWarning { .. } | RegRejected { .. } => DriveProblem { reason: ProblemReason::Removed },
            Erasing { game, label, .. } => BurnFailed { game, label, reason: BurnFailure::DriveRemoved, cdrw: true },
            Burning { game, label } => BurnFailed { game: Some(game), label: Some(label), reason: BurnFailure::DriveRemoved, cdrw: self.is_cdrw() },
            Verifying { game } => BurnFailed { game: Some(game), label: None, reason: BurnFailure::DriveRemoved, cdrw: self.is_cdrw() },
            s => s, // Launching, LaunchError, BurnDone, BurnFailed, DriveProblem, CatalogError, Boot
        };
    }

    fn is_cdrw(&self) -> bool {
        self.last_media.as_ref().is_some_and(|m| m.physical == Physical::CdRw)
    }

    // ---------- jogar ----------

    fn on_auto_insert(&mut self) {
        self.begin(DriveOp::Read, Pending::Read(ReadThen::AutoInsert));
    }

    fn after_auto_read(&mut self, m: &MediaInfo) {
        if self.state != State::Library {
            return; // o usuário já foi para outra tela durante a leitura
        }
        let class = classify(m, &self.catalog, None);
        match class {
            MediaClass::Known { game } => match self.catalog.settings.on_disc_insert {
                OnDiscInsert::Focus => {
                    self.focus_hint = Some(game);
                    self.set_toast(Toast::Focus { game });
                }
                OnDiscInsert::Launch => self.start_launch(game),
            },
            MediaClass::Unknown { .. } => self.set_toast(Toast::Unknown),
            MediaClass::ReadError => self.set_toast(Toast::ReadError),
            _ => self.set_toast(Toast::NotAGame),
        }
    }

    fn identify(&mut self, game: GameId) {
        self.state = State::Reading { game };
        self.begin(DriveOp::Read, Pending::Read(ReadThen::Identify(game)));
    }

    fn after_identify(&mut self, game: GameId, m: &MediaInfo) {
        if self.state != (State::Reading { game }) {
            return;
        }
        let class = classify(m, &self.catalog, Some(game));
        if class == MediaClass::Match {
            self.ficha_since = self.now;
            self.state = State::Identified { game };
        } else {
            self.state = State::Rejected { game, class };
        }
    }

    fn start_launch(&mut self, game: GameId) {
        let Some(g) = self.catalog.game(game) else { return };
        let req = request_for(g);
        self.armed = false; // qualquer lançamento desarma a mídia presente
        match self.launcher.launch(&req) {
            Ok(()) => self.state = State::Launching { game, started: self.now, min_ms: self.catalog.settings.loading_min_ms },
            Err(reason) => self.state = State::LaunchError { game, reason },
        }
    }

    fn select(&mut self, id: GameId) -> Result<(), IntentError> {
        let Some(game) = self.catalog.game(id) else { return Err(IntentError::Ignored) };
        if game.discs.is_empty() {
            self.state = State::NoDiscYet { game: id };
            return Ok(());
        }
        if let Some(reason) = self.drive_problem() {
            self.state = State::DriveProblem { reason };
            return Ok(());
        }
        if self.media_present {
            self.identify(id);
        } else {
            let tray = self.open_tray();
            self.state = State::WaitingDisc { game: id, tray };
        }
        Ok(())
    }

    // ---------- cadastro ----------

    fn start_reg(&mut self, game: Option<GameId>) {
        if let Some(reason) = self.drive_problem() {
            self.state = State::DriveProblem { reason };
        } else if !self.caps().can_write() {
            self.state = State::DriveProblem { reason: ProblemReason::CannotBurn };
        } else if self.media_present {
            self.reg_read(game);
        } else {
            let tray = self.open_tray();
            self.state = State::RegInsert { game, tray };
        }
    }

    fn reg_read(&mut self, game: Option<GameId>) {
        self.state = State::RegReading { game };
        self.begin(DriveOp::Read, Pending::Read(ReadThen::Reg(game)));
    }

    fn after_reg_read(&mut self, game: Option<GameId>, m: &MediaInfo) {
        if self.state != (State::RegReading { game }) {
            return;
        }
        let m = m.clone();
        let caps = self.caps();
        let reject = |reason| State::RegRejected { game, reason };
        let has_ini = m.game_ini.is_some();
        let next = match (m.kind, m.physical) {
            (MediaKind::Unreadable, _) => reject(RegRejectReason::ReadError),
            (MediaKind::Audio, _) | (MediaKind::Mixed, _) if !has_ini => reject(RegRejectReason::Audio),
            (MediaKind::Blank, Physical::CdRom) => reject(RegRejectReason::NotWritable),
            (MediaKind::Blank, Physical::CdRw) if caps.write_cdrw == Tri::No => reject(RegRejectReason::NotWritable),
            (MediaKind::Blank, Physical::CdR) if caps.write_cdr == Tri::No => reject(RegRejectReason::NotWritable),
            (MediaKind::Blank, _) => self.after_blank(game),
            (_, Physical::CdRw) if caps.erase != Tri::No && caps.write_cdrw != Tri::No => {
                let old = m.ini().map(|i| i.id).filter(|id| self.catalog.game_of_disc(*id).is_some());
                State::RegEraseConfirm { game, step: 1, old_disc: old }
            }
            (_, Physical::CdR) => reject(RegRejectReason::CdrUsed),
            _ => reject(RegRejectReason::NotWritable),
        };
        self.state = next;
    }

    fn after_blank(&self, game: Option<GameId>) -> State {
        match game.and_then(|g| self.catalog.game(g)) {
            Some(g) => State::RegLabelPreview { game: g.game_id, label: gameini::sanitize_label(&g.name) },
            None => State::RegChooseGame,
        }
    }

    fn do_erase(&mut self, game: Option<GameId>, label: Option<String>, old: Option<DiscId>) {
        self.state = State::Erasing { game, label: label.clone(), old_disc: old };
        self.progress = Some(0);
        self.begin(DriveOp::Erase { quick: true }, Pending::Erase { game, label, old });
    }

    fn after_erase(&mut self, game: Option<GameId>, label: Option<String>, old: Option<DiscId>, res: Result<(), DriveError>) {
        if !matches!(self.state, State::Erasing { .. }) {
            return;
        }
        match res {
            Ok(()) => {
                // Só agora a chave antiga deixa de existir (BURNING.md).
                if let Some(o) = old {
                    self.catalog.remove_disc(o);
                    self.persist();
                }
                self.last_media = Some(MediaInfo { kind: MediaKind::Blank, physical: Physical::CdRw, label: None, game_ini: None });
                self.state = match (game, label) {
                    (Some(g), Some(l)) if self.catalog.game(g).is_some() => State::RegLabelPreview { game: g, label: l },
                    _ => self.after_blank(game),
                };
            }
            Err(_) => self.state = State::BurnFailed { game, label, reason: BurnFailure::EraseError, cdrw: true },
        }
    }

    fn do_burn(&mut self, game: GameId, label: String) {
        let Some(name) = self.catalog.game(game).map(|g| g.name.clone()) else { return };
        if self.drive_id.is_none() {
            return;
        }
        let physical = self.last_media.as_ref().map_or(Physical::Unknown, |m| m.physical);
        let cdrw = physical == Physical::CdRw;
        let disc_id = Uuid::new_v4();
        let spec = DiscImageSpec { label: label.clone(), game_ini: gameini::write(disc_id, &name) };

        self.state = State::Burning { game, label: label.clone() };
        self.progress = Some(0);
        self.begin(DriveOp::Burn(spec), Pending::Burn(BurnCtx { game, label, disc_id, cdrw, physical }));
    }

    fn after_burn(&mut self, b: BurnCtx, res: Result<(), DriveError>) {
        if !matches!(self.state, State::Burning { .. }) {
            return;
        }
        if let Err(e) = res {
            let reason = if e == DriveError::Removed { BurnFailure::DriveRemoved } else { BurnFailure::WriteError };
            self.state = State::BurnFailed { game: Some(b.game), label: Some(b.label), reason, cdrw: b.cdrw };
            return;
        }
        self.state = State::Verifying { game: b.game };
        self.begin(DriveOp::Read, Pending::Verify(b));
    }

    fn after_verify(&mut self, b: BurnCtx, res: Result<MediaInfo, DriveError>) {
        if !matches!(self.state, State::Verifying { .. }) {
            return;
        }
        let BurnCtx { game, label, disc_id, cdrw, physical } = b;
        let verified = match &res {
            Ok(m) => m.ini().is_some_and(|i| i.id == disc_id),
            Err(_) => false,
        };
        if !verified {
            let reason = if res == Err(DriveError::Removed) { BurnFailure::DriveRemoved } else { BurnFailure::VerifyMismatch };
            self.state = State::BurnFailed { game: Some(game), label: Some(label), reason, cdrw };
            return;
        }
        let media = if self.drive.kind() == BackendKind::Fake {
            DiscMedia::Iso
        } else {
            match physical {
                Physical::CdR => DiscMedia::CdR,
                Physical::CdRw => DiscMedia::CdRw,
                _ => DiscMedia::Unknown,
            }
        };
        // O catálogo só recebe o disc_id depois da verificação (invariante de BURNING.md).
        let disc = Disc { disc_id, label, media, created_at: self.now, origin: DiscOrigin::Burned, note: None };
        match self.catalog.add_disc(game, disc) {
            Ok(()) => {
                self.persist();
                self.focus_hint = Some(game);
                self.state = State::BurnDone { game };
            }
            Err(_) => self.state = State::BurnFailed { game: Some(game), label: None, reason: BurnFailure::WriteError, cdrw },
        }
    }

    // ---------- intenções ----------

    pub fn dispatch(&mut self, intent: Intent, now: u64) -> Result<(), IntentError> {
        self.now = now;
        let ignored = Err(IntentError::Ignored);
        match intent {
            Intent::Select(id) => {
                if self.state != State::Library {
                    return ignored;
                }
                self.select(id)
            }
            Intent::AddGame => {
                if self.state != State::Library {
                    return ignored;
                }
                self.start_reg(None);
                Ok(())
            }
            Intent::Options(id) => {
                if self.state != State::Library || self.catalog.game(id).is_none() {
                    return ignored;
                }
                self.state = State::GameOptions { game: id };
                Ok(())
            }
            Intent::OpenSettings => {
                if self.state != State::Library {
                    return ignored;
                }
                self.state = State::Settings;
                Ok(())
            }
            Intent::Back => self.back(),
            Intent::Action(a) => self.action(a),
            Intent::HoldComplete => match self.state.clone() {
                State::RegEraseConfirm { game, step: 2, old_disc } => {
                    let label = game.and_then(|g| self.catalog.game(g)).map(|g| gameini::sanitize_label(&g.name));
                    self.do_erase(game, label, old_disc);
                    Ok(())
                }
                _ => ignored,
            },
            Intent::ChooseGame(id) => {
                if self.state != State::RegChooseGame || self.catalog.game(id).is_none() {
                    return ignored;
                }
                self.state = self.after_blank(Some(id));
                Ok(())
            }
            Intent::CreateGame(n) => {
                if self.state != State::RegChooseGame {
                    return ignored;
                }
                let id = self.catalog.add_game(Game::new(n.name.trim(), n.kind)).map_err(IntentError::Invalid)?;
                self.persist();
                self.state = self.after_blank(Some(id));
                Ok(())
            }
            Intent::LabelEdit(text) => match self.state.clone() {
                State::RegLabelPreview { game, .. } => {
                    self.state = State::RegLabelPreview { game, label: gameini::sanitize_label(&text) };
                    Ok(())
                }
                _ => ignored,
            },
            Intent::RenameGame(id, name) => match self.state {
                State::GameOptions { game } if game == id => {
                    self.catalog.rename_game(id, &name).map_err(IntentError::Invalid)?;
                    self.persist();
                    Ok(())
                }
                _ => ignored,
            },
            Intent::UpdateGame(id, n) => match self.state {
                State::GameOptions { game } if game == id => {
                    self.catalog.update_game(id, &n.name, n.kind).map_err(IntentError::Invalid)?;
                    self.persist();
                    Ok(())
                }
                _ => ignored,
            },
            Intent::SetSetting(s) => {
                if self.state != State::Settings {
                    return ignored;
                }
                let st = &mut self.catalog.settings;
                match s {
                    SettingChange::Drive(d) => st.drive = d,
                    SettingChange::OnDiscInsert(v) => st.on_disc_insert = v,
                    SettingChange::LoadingMinMs(v) => st.loading_min_ms = v,
                    SettingChange::Locale(v) => st.locale = v,
                    SettingChange::WindowMode(v) => st.window_mode = v,
                    SettingChange::CoversOnline(v) => st.covers_online_enabled = v,
                }
                self.persist();
                self.select_drive();
                Ok(())
            }
        }
    }

    fn back(&mut self) -> Result<(), IntentError> {
        use State::*;
        if matches!(self.state, Reading { .. } | RegReading { .. }) {
            self.pending = None; // desistiu da leitura em curso
        }
        let next = match self.state.clone() {
            NoDiscYet { .. } | WaitingDisc { .. } | Reading { .. } | Rejected { .. } | LaunchError { .. } | GameOptions { .. } | Settings | DriveProblem { .. }
            | RegInsert { .. } | RegReading { .. } | RegChooseGame | BurnFailed { .. } | RegRejected { .. } => Library,
            BurnDone { game } => {
                self.focus_hint = Some(game);
                Library
            }
            AdoptConfirm { game, class, .. } => Rejected { game, class },
            RemoveConfirm { game } => GameOptions { game },
            RegEraseConfirm { game, .. } => {
                self.eject();
                let tray = self.tray_after_removal();
                RegInsert { game, tray }
            }
            RegLabelPreview { .. } => RegChooseGame,
            RegCdrWarning { game, label } => RegLabelPreview { game, label },
            _ => return Err(IntentError::Ignored), // Boot, CatalogError, Library, Identified, Launching, Erasing, Burning, Verifying
        };
        self.state = next;
        Ok(())
    }

    fn action(&mut self, a: Action) -> Result<(), IntentError> {
        use State::*;
        let ignored = Err(IntentError::Ignored);
        match (self.state.clone(), a) {
            (NoDiscYet { game }, Action::Burn) => self.start_reg(Some(game)),
            (WaitingDisc { game, .. }, Action::RetryTray) if self.caps().tray_open == Tri::Yes => {
                let tray = self.open_tray();
                self.state = WaitingDisc { game, tray };
            }
            (Rejected { game, .. }, Action::TryOther) => {
                self.eject();
                let tray = self.tray_after_removal();
                self.state = WaitingDisc { game, tray };
            }
            (Rejected { game, class: class @ MediaClass::Unknown { .. } }, Action::Adopt) => {
                let Some(ini) = self.last_media.as_ref().and_then(|m| m.ini()) else { return ignored };
                self.state = AdoptConfirm { game, disc: ini.id, ini_name: ini.name, class };
            }
            (Rejected { class: MediaClass::OtherGame { other }, .. }, Action::PlayOther) => self.start_launch(other),
            (AdoptConfirm { game, disc, .. }, Action::Confirm) => {
                let media = match self.last_media.as_ref().map(|m| m.physical) {
                    _ if self.drive.kind() == BackendKind::Fake => DiscMedia::Iso,
                    Some(Physical::CdR) => DiscMedia::CdR,
                    Some(Physical::CdRw) => DiscMedia::CdRw,
                    _ => DiscMedia::Unknown,
                };
                let label = self.last_media.as_ref().and_then(|m| m.label.clone()).unwrap_or_default();
                let d = Disc { disc_id: disc, label, media, created_at: self.now, origin: DiscOrigin::Adopted, note: None };
                self.catalog.add_disc(game, d).map_err(IntentError::Invalid)?;
                self.persist();
                self.start_launch(game);
            }
            (LaunchError { .. }, Action::Confirm) => self.state = Library,
            (DriveProblem { .. }, Action::ChooseDrive) => self.state = Settings,
            (GameOptions { game }, Action::BurnAnother) => self.start_reg(Some(game)),
            (GameOptions { game }, Action::RemoveGame) => self.state = RemoveConfirm { game },
            (GameOptions { game }, Action::UnlinkDisc(d)) => {
                if !self.catalog.game(game).is_some_and(|g| g.discs.iter().any(|x| x.disc_id == d)) {
                    return ignored;
                }
                self.catalog.remove_disc(d);
                self.persist();
            }
            (RemoveConfirm { game }, Action::Confirm) => {
                self.catalog.remove_game(game);
                self.persist();
                self.state = Library;
            }
            (RegInsert { .. }, Action::RetryTray) if self.caps().tray_open == Tri::Yes => {
                let g = if let RegInsert { game, .. } = self.state { game } else { None };
                let tray = self.open_tray();
                self.state = RegInsert { game: g, tray };
            }
            (RegEraseConfirm { game, step: 1, old_disc }, Action::Confirm) => self.state = RegEraseConfirm { game, step: 2, old_disc },
            (RegLabelPreview { game, label }, Action::Continue) => {
                let cdr = self.last_media.as_ref().is_some_and(|m| m.physical == Physical::CdR);
                if cdr {
                    self.state = RegCdrWarning { game, label };
                } else {
                    self.do_burn(game, label);
                }
            }
            (RegCdrWarning { game, label }, Action::Burn) => self.do_burn(game, label),
            (BurnDone { game }, Action::Done) => {
                self.focus_hint = Some(game);
                self.state = Library;
            }
            (BurnFailed { game, label, cdrw: true, .. }, Action::EraseRetry) => self.do_erase(game, label, None),
            (BurnFailed { game, .. }, Action::TryOther) | (RegRejected { game, .. }, Action::TryOther) => {
                self.eject();
                let tray = self.tray_after_removal();
                self.state = RegInsert { game, tray };
            }
            (CatalogError { .. }, Action::StartEmpty) => {
                if let Some(p) = self.catalog_path.clone() {
                    self.catalog = start_empty(&p).map_err(|_| IntentError::Ignored)?;
                } else {
                    self.catalog = Catalog::default();
                }
                self.state = Library;
            }
            (CatalogError { backups }, Action::RestoreBackup(i)) if i < backups => {
                let Some(p) = self.catalog_path.clone() else { return ignored };
                self.catalog = restore_backup(&p, i).map_err(|_| IntentError::Ignored)?;
                self.state = Library;
            }
            _ => return ignored,
        }
        Ok(())
    }

    /// Ações válidas no estado atual, na ordem de exibição; a padrão vem marcada (UI-CONTRACT).
    pub fn actions(&mut self) -> Vec<(Action, bool)> {
        use State::*;
        let tray = self.caps().tray_open == Tri::Yes;
        match self.state.clone() {
            NoDiscYet { .. } => vec![(Action::Burn, true)],
            WaitingDisc { .. } | RegInsert { .. } if tray => vec![(Action::RetryTray, false)],
            Rejected { class, .. } => {
                let mut v = vec![(Action::TryOther, true)];
                match class {
                    MediaClass::Unknown { .. } => v.push((Action::Adopt, false)),
                    MediaClass::OtherGame { .. } => v.push((Action::PlayOther, false)),
                    _ => {}
                }
                v
            }
            AdoptConfirm { .. } | LaunchError { .. } | RemoveConfirm { .. } => vec![(Action::Confirm, true)],
            DriveProblem { .. } => vec![(Action::ChooseDrive, true)],
            GameOptions { game } => {
                let mut v = vec![(Action::BurnAnother, false), (Action::RemoveGame, false)];
                if let Some(g) = self.catalog.game(game) {
                    v.extend(g.discs.iter().map(|d| (Action::UnlinkDisc(d.disc_id), false)));
                }
                v
            }
            RegEraseConfirm { step: 1, .. } => vec![(Action::Confirm, true)],
            RegLabelPreview { .. } => vec![(Action::Continue, true)],
            RegCdrWarning { .. } => vec![(Action::Burn, true)],
            BurnDone { .. } => vec![(Action::Done, true)],
            BurnFailed { cdrw: true, .. } => vec![(Action::EraseRetry, true), (Action::TryOther, false)],
            BurnFailed { .. } | RegRejected { .. } => vec![(Action::TryOther, true)],
            CatalogError { backups } => {
                let mut v: Vec<_> = (0..backups).map(|i| (Action::RestoreBackup(i), i == 0)).collect();
                v.push((Action::StartEmpty, backups == 0));
                v
            }
            _ => vec![],
        }
    }
}
