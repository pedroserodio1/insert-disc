//! Instantâneo de estado enviado à UI (docs/UI-CONTRACT.md). Só códigos, nunca texto visível.

use serde::Serialize;
use serde_json::json;

use crate::app::*;
use crate::catalog::*;
use crate::classify::MediaClass;
use crate::drive::*;
use crate::launch::Launcher;

pub const CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct GameView {
    pub game_id: GameId,
    pub name: String,
    pub kind: &'static str,
    pub cover: Option<String>,
    pub spine_color: Option<String>,
    pub disc_count: usize,
    /// Campos do tipo do jogo (para a edição); vazios quando não se aplicam.
    pub app_id: Option<u32>,
    pub executable: Option<String>,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub requires_elevation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscView {
    pub disc_id: DiscId,
    pub label: String,
    pub created_at: u64,
    pub origin: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaView {
    pub label: Option<String>,
    pub physical: &'static str,
    pub disc_id: Option<DiscId>,
    pub ini_name: Option<String>,
    pub class: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionView {
    pub id: String,
    pub default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Timing {
    pub min_ms: u32,
    pub started_at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToastView {
    pub id: u64,
    pub code: &'static str,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub version: u32,
    pub state: &'static str,
    pub game: Option<GameView>,
    pub other_game: Option<GameView>,
    /// Discos do jogo em contexto (tela de opções).
    pub game_discs: Vec<DiscView>,
    pub media: Option<MediaView>,
    pub reason: Option<String>,
    /// Rótulo em edição/gravação (REG_LABEL_PREVIEW e seguintes).
    pub label: Option<String>,
    /// Passo da confirmação de apagar (1 ou 2).
    pub step: Option<u8>,
    pub actions: Vec<ActionView>,
    pub progress: Option<u8>,
    pub timing: Option<Timing>,
    pub capabilities: serde_json::Value,
    pub drive: &'static str,
    pub library: Vec<GameView>,
    pub settings: Settings,
    /// Entregue uma única vez; a UI aplica e ignora repetições.
    pub focus_hint: Option<GameId>,
    pub toast: Option<ToastView>,
}

fn view(g: &Game) -> GameView {
    let (kind, app_id, executable, args, working_dir, requires_elevation) = match &g.kind {
        GameKind::Steam { app_id } => ("steam", Some(*app_id), None, Vec::new(), None, false),
        GameKind::Custom { executable, args, working_dir, requires_elevation } => (
            "custom",
            None,
            Some(executable.to_string_lossy().into_owned()),
            args.clone(),
            working_dir.as_ref().map(|w| w.to_string_lossy().into_owned()),
            *requires_elevation,
        ),
    };
    GameView {
        game_id: g.game_id,
        name: g.name.clone(),
        kind,
        cover: g.cover.clone(),
        spine_color: g.spine_color.clone(),
        disc_count: g.discs.len(),
        app_id,
        executable,
        args,
        working_dir,
        requires_elevation,
    }
}

fn tri(t: Tri) -> &'static str {
    match t {
        Tri::Yes => "yes",
        Tri::No => "no",
        Tri::Unknown => "unknown",
    }
}

fn action_id(a: &Action) -> String {
    match a {
        Action::PlayOther => "play_other".into(),
        Action::TryOther => "try_other".into(),
        Action::Adopt => "adopt".into(),
        Action::Burn => "burn".into(),
        Action::Confirm => "confirm".into(),
        Action::Continue => "continue".into(),
        Action::Done => "done".into(),
        Action::RetryTray => "retry_tray".into(),
        Action::EraseRetry => "erase_retry".into(),
        Action::ChooseDrive => "choose_drive".into(),
        Action::BurnAnother => "burn_another".into(),
        Action::RemoveGame => "remove_game".into(),
        Action::UnlinkDisc(d) => format!("unlink_disc:{d}"),
        Action::StartEmpty => "start_empty".into(),
        Action::RestoreBackup(i) => format!("restore_backup:{i}"),
    }
}

fn state_game(s: &State) -> Option<GameId> {
    use State::*;
    match s {
        NoDiscYet { game } | WaitingDisc { game, .. } | Reading { game } | Identified { game } | Rejected { game, .. } | AdoptConfirm { game, .. }
        | Launching { game, .. } | LaunchError { game, .. } | GameOptions { game } | RemoveConfirm { game } | RegLabelPreview { game, .. }
        | RegCdrWarning { game, .. } | Burning { game, .. } | Verifying { game } | BurnDone { game } => Some(*game),
        RegInsert { game, .. } | RegReading { game } | RegEraseConfirm { game, .. } | Erasing { game, .. } | BurnFailed { game, .. } | RegRejected { game, .. } => *game,
        _ => None,
    }
}

fn reason(s: &State) -> Option<String> {
    use State::*;
    Some(match s {
        Rejected { class, .. } | AdoptConfirm { class, .. } => class.key().into(),
        LaunchError { reason, .. } => reason.key().into(),
        DriveProblem { reason } => match reason {
            ProblemReason::NoDrive => "no_drive",
            ProblemReason::Removed => "removed",
            ProblemReason::CannotBurn => "cannot_burn",
        }
        .into(),
        BurnFailed { reason, cdrw, .. } => match (reason, cdrw) {
            (BurnFailure::WriteError, false) => "cdr_lost",
            (BurnFailure::WriteError, true) => "write_error",
            (BurnFailure::VerifyMismatch, _) => "verify_mismatch",
            (BurnFailure::DriveRemoved, _) => "drive_removed",
            (BurnFailure::EraseError, _) => "erase_error",
        }
        .into(),
        RegRejected { reason, .. } => match reason {
            RegRejectReason::CdrUsed => "CDR_USED",
            RegRejectReason::NotWritable => "NOT_WRITABLE",
            RegRejectReason::Audio => "AUDIO",
            RegRejectReason::ReadError => "READ_ERROR",
        }
        .into(),
        WaitingDisc { tray, .. } | RegInsert { tray, .. } => match tray {
            TrayHint::Opening => "tray_opening",
            TrayHint::Manual => "tray_manual",
            TrayHint::Failed => "tray_failed",
        }
        .into(),
        _ => return None,
    })
}

impl<D: DriveBackend, L: Launcher> App<D, L> {
    pub fn snapshot(&mut self) -> Snapshot {
        let actions = self.actions().iter().map(|(a, d)| ActionView { id: action_id(a), default: *d }).collect();
        let caps = self.caps();
        let state = self.state().clone();

        let game_id = state_game(&state);
        let other = match &state {
            State::Rejected { class: MediaClass::OtherGame { other }, .. } => Some(*other),
            _ => None,
        };
        let shows_media = matches!(
            state,
            State::Identified { .. } | State::Rejected { .. } | State::AdoptConfirm { .. } | State::RegEraseConfirm { .. } | State::RegRejected { .. }
        );
        let class = match &state {
            State::Rejected { class, .. } | State::AdoptConfirm { class, .. } => Some(class.key()),
            State::Identified { .. } => Some("MATCH"),
            _ => None,
        };
        let media = self.last_media.as_ref().filter(|_| shows_media).map(|m| {
            let ini = m.ini();
            MediaView {
                label: m.label.clone(),
                physical: match m.physical {
                    Physical::CdRom => "cd-rom",
                    Physical::CdR => "cd-r",
                    Physical::CdRw => "cd-rw",
                    Physical::Unknown => "unknown",
                },
                disc_id: ini.as_ref().map(|i| i.id),
                ini_name: ini.and_then(|i| i.name),
                class,
            }
        });
        let timing = match state {
            State::Launching { started, min_ms, .. } => Some(Timing { min_ms, started_at: started }),
            _ => None,
        };
        let toast = self.toast.as_ref().map(|(id, t)| {
            let (code, params) = match t {
                Toast::Focus { game } => ("focus", json!({ "other": self.catalog.game(*game).map(|g| g.name.clone()) })),
                Toast::Unknown => ("unknown", json!({})),
                Toast::NotAGame => ("not_a_game", json!({})),
                Toast::ReadError => ("read_error", json!({})),
                Toast::DriveRemoved => ("drive_removed", json!({})),
            };
            ToastView { id: *id, code, params }
        });

        Snapshot {
            version: CONTRACT_VERSION,
            state: state.name(),
            game: game_id.and_then(|g| self.catalog.game(g)).map(view),
            other_game: other.and_then(|g| self.catalog.game(g)).map(view),
            game_discs: game_id
                .and_then(|g| self.catalog.game(g))
                .map(|g| g.discs.iter().map(|d| DiscView { disc_id: d.disc_id, label: d.label.clone(), created_at: d.created_at, origin: match d.origin { DiscOrigin::Burned => "burned", DiscOrigin::Adopted => "adopted" } }).collect())
                .unwrap_or_default(),
            media,
            reason: reason(&state),
            label: match &state {
                State::RegLabelPreview { label, .. } | State::RegCdrWarning { label, .. } | State::Burning { label, .. } => Some(label.clone()),
                State::BurnFailed { label, .. } | State::Erasing { label, .. } => label.clone(),
                _ => None,
            },
            step: match &state {
                State::RegEraseConfirm { step, .. } => Some(*step),
                _ => None,
            },
            actions,
            progress: self.progress.filter(|_| matches!(state, State::Burning { .. } | State::Verifying { .. } | State::BurnDone { .. } | State::Erasing { .. })),
            timing,
            capabilities: json!({
                "tray_open": tri(caps.tray_open), "tray_close": tri(caps.tray_close), "eject": tri(caps.eject),
                "write_cdr": tri(caps.write_cdr), "write_cdrw": tri(caps.write_cdrw), "erase": tri(caps.erase),
                "media_events": tri(caps.media_events),
            }),
            drive: match self.drive_status {
                DriveStatus::Ok => "ok",
                DriveStatus::None => "none",
                DriveStatus::Removed => "removed",
            },
            library: self.catalog.sorted_games().into_iter().map(view).collect(),
            settings: self.catalog.settings.clone(),
            focus_hint: self.focus_hint.take(),
            toast,
        }
    }
}
