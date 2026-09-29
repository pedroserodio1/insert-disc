//! Interface `DriveBackend` (docs/DRIVE-LAYER.md). O núcleo só fala com isto.

use crate::gameini::GameIni;

pub type DriveId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveInfo {
    pub id: DriveId,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    Yes,
    No,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub tray_open: Tri,
    pub tray_close: Tri,
    pub eject: Tri,
    pub write_cdr: Tri,
    pub write_cdrw: Tri,
    pub erase: Tri,
    pub media_events: Tri,
}

impl Capabilities {
    pub const ALL: Capabilities = Capabilities {
        tray_open: Tri::Yes,
        tray_close: Tri::Yes,
        eject: Tri::Yes,
        write_cdr: Tri::Yes,
        write_cdrw: Tri::Yes,
        erase: Tri::Yes,
        media_events: Tri::Yes,
    };

    pub fn can_write(&self) -> bool {
        self.write_cdr != Tri::No || self.write_cdrw != Tri::No
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Blank,
    Data,
    Audio,
    Mixed,
    Unreadable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Physical {
    CdRom,
    CdR,
    CdRw,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaInfo {
    pub kind: MediaKind,
    pub physical: Physical,
    pub label: Option<String>,
    /// Bytes crus do `GAME.INI` (já limitados pelo backend), se houver.
    pub game_ini: Option<Vec<u8>>,
}

impl MediaInfo {
    pub fn unreadable() -> Self {
        MediaInfo { kind: MediaKind::Unreadable, physical: Physical::Unknown, label: None, game_ini: None }
    }

    pub fn ini(&self) -> Option<GameIni> {
        self.game_ini.as_deref().and_then(|b| crate::gameini::parse(b).ok())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveEvent {
    MediaArrived,
    MediaRemoved,
    DriveAdded,
    DriveRemoved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveError {
    Unsupported,
    NoMedia,
    Removed,
    Io(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Real,
    Fake,
}

/// Só o que vai para o disco (docs/DISC-FORMAT.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscImageSpec {
    pub label: String,
    pub game_ini: String,
}

/// Operação longa do drive (A5). Backends reais a executam numa thread própria e devolvem o
/// resultado por `poll_ops`; a UI continua respondendo (o `App` nunca bloqueia).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveOp {
    Read,
    Burn(DiscImageSpec),
    Erase { quick: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpResult {
    Read(Result<MediaInfo, DriveError>),
    Burn(Result<(), DriveError>),
    Erase(Result<(), DriveError>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpUpdate {
    /// Progresso 0..=100 da gravação ou do apagamento em curso.
    Progress(u8),
    Done(OpResult),
}

pub trait DriveBackend {
    fn kind(&self) -> BackendKind;
    fn list_drives(&mut self) -> Vec<DriveInfo>;
    fn capabilities(&mut self, drive: &str) -> Capabilities;
    fn media_present(&mut self, drive: &str) -> bool;
    fn open_tray(&mut self, drive: &str) -> Result<(), DriveError>;
    fn close_tray(&mut self, drive: &str) -> Result<(), DriveError>;
    fn eject(&mut self, drive: &str) -> Result<(), DriveError>;
    fn read_media(&mut self, drive: &str) -> Result<MediaInfo, DriveError>;
    fn burn(&mut self, drive: &str, spec: &DiscImageSpec, progress: &mut dyn FnMut(u8)) -> Result<(), DriveError>;
    fn erase(&mut self, drive: &str, quick: bool, progress: &mut dyn FnMut(u8)) -> Result<(), DriveError>;
    /// Inicia uma operação (uma por vez). Não bloqueia: o resultado chega por `poll_ops`.
    fn start_op(&mut self, drive: &str, op: DriveOp, now: u64);
    /// Progresso e resultados desde a última chamada (`now` em ms, para backends simulados).
    fn poll_ops(&mut self, now: u64) -> Vec<OpUpdate>;
    /// A unidade escolhida: os eventos de mídia (`MediaArrived`/`MediaRemoved`) valem só para ela.
    /// Backends com uma única unidade (o falso) ignoram.
    fn watch(&mut self, _drive: &str) {}
    /// Eventos acumulados desde a última chamada.
    fn poll_events(&mut self) -> Vec<DriveEvent>;
}
