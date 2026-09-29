//! `WindowsDrive`, parte de leitura (C1, docs/DRIVE-LAYER.md): lista unidades ópticas, detecta
//! inserção e remoção por **polling** e lê rótulo e `GAME.INI` do volume montado. Funciona igual
//! com um drive físico e com uma ISO montada (`Mount-DiskImage`), que aparece como uma unidade nova.
//!
//! Ainda sem hardware (W2 físico, W3, W4, W5):
//! - tipo físico (CD-R/CD-RW/ROM) sai como `Unknown`; mídia virgem aparece como ilegível;
//! - gaveta, ejeção, gravação e apagamento não são suportados (`Unsupported`);
//! - o gatilho é só polling (o `WM_DEVICECHANGE` do W2 chega ~1,2 s depois e sem `DBTF_MEDIA`, então
//!   não ajuda mais que o polling). ponytail: acrescentar o gatilho por janela oculta se o drive
//!   físico mostrar latência ruim; o polling de 750 ms mantém a leitura do volume a cada ciclo.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use insert_disc_core::drive::*;
use insert_disc_core::gameini::MAX_INI_BYTES;

const POLL: Duration = Duration::from_millis(750);

enum Op {
    Thread(mpsc::Receiver<OpResult>),
    Ready(OpResult),
}

struct Seen {
    drives: Vec<String>,
    media: bool,
}

#[derive(Default)]
pub struct WindowsDrive {
    watched: Option<String>,
    seen: Option<Seen>,
    last_poll: Option<Instant>,
    events: Vec<DriveEvent>,
    op: Option<Op>,
}

impl WindowsDrive {
    pub fn new() -> Self {
        WindowsDrive::default()
    }

    fn sample(&self) -> Seen {
        Seen { drives: optical_drives(), media: self.watched.as_deref().is_some_and(volume_ready) }
    }
}

/// Classifica o conteúdo da raiz de um volume. `label` vem do sistema (ou `None`).
pub fn read_root(root: &Path, label: Option<String>) -> MediaInfo {
    let Ok(rd) = fs::read_dir(root) else { return MediaInfo::unreadable() };
    let names: Vec<String> = rd.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
    let lower: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
    let (kind, game_ini) = if names.is_empty() {
        (MediaKind::Blank, None)
    } else if lower.iter().all(|n| n.ends_with(".cda")) {
        (MediaKind::Audio, None)
    } else {
        let ini = names.iter().find(|n| n.eq_ignore_ascii_case("GAME.INI")).and_then(|n| {
            let mut buf = Vec::new();
            fs::File::open(root.join(n)).ok()?.take(MAX_INI_BYTES as u64 + 1).read_to_end(&mut buf).ok()?;
            Some(buf)
        });
        (MediaKind::Data, ini)
    };
    MediaInfo { kind, physical: Physical::Unknown, label, game_ini }
}

fn read_volume(drive: &str) -> Result<MediaInfo, DriveError> {
    if !volume_ready(drive) {
        return Err(DriveError::NoMedia);
    }
    Ok(read_root(Path::new(&format!("{drive}\\")), volume_label(drive)))
}

impl DriveBackend for WindowsDrive {
    fn kind(&self) -> BackendKind {
        BackendKind::Real
    }

    fn list_drives(&mut self) -> Vec<DriveInfo> {
        optical_drives()
            .into_iter()
            .map(|id| {
                let name = match volume_label(&id) {
                    Some(l) if !l.is_empty() => format!("{id} ({l})"),
                    Some(_) => id.clone(),
                    None => format!("{id} (sem disco)"),
                };
                DriveInfo { id, name }
            })
            .collect()
    }

    fn capabilities(&mut self, _: &str) -> Capabilities {
        Capabilities {
            tray_open: Tri::No,
            tray_close: Tri::No,
            eject: Tri::No,
            // sem hardware para validar o IMAPI2: "não sei", para o fluxo de cadastro não travar antes da hora
            write_cdr: Tri::Unknown,
            write_cdrw: Tri::Unknown,
            erase: Tri::No,
            media_events: Tri::Unknown,
        }
    }

    fn media_present(&mut self, drive: &str) -> bool {
        volume_ready(drive)
    }

    fn open_tray(&mut self, _: &str) -> Result<(), DriveError> {
        Err(DriveError::Unsupported)
    }
    fn close_tray(&mut self, _: &str) -> Result<(), DriveError> {
        Err(DriveError::Unsupported)
    }
    fn eject(&mut self, _: &str) -> Result<(), DriveError> {
        Err(DriveError::Unsupported)
    }

    fn read_media(&mut self, drive: &str) -> Result<MediaInfo, DriveError> {
        read_volume(drive)
    }
    fn burn(&mut self, _: &str, _: &DiscImageSpec, _: &mut dyn FnMut(u8)) -> Result<(), DriveError> {
        Err(DriveError::Unsupported)
    }
    fn erase(&mut self, _: &str, _: bool, _: &mut dyn FnMut(u8)) -> Result<(), DriveError> {
        Err(DriveError::Unsupported)
    }

    /// Leitura numa thread (um drive lento não trava a UI); gravar e apagar ainda não existem.
    fn start_op(&mut self, drive: &str, op: DriveOp, _now: u64) {
        self.op = Some(match op {
            DriveOp::Read => {
                let (tx, rx) = mpsc::channel();
                let drive = drive.to_string();
                std::thread::spawn(move || {
                    let _ = tx.send(OpResult::Read(read_volume(&drive)));
                });
                Op::Thread(rx)
            }
            DriveOp::Burn(_) => Op::Ready(OpResult::Burn(Err(DriveError::Unsupported))),
            DriveOp::Erase { .. } => Op::Ready(OpResult::Erase(Err(DriveError::Unsupported))),
        });
    }

    fn poll_ops(&mut self, _now: u64) -> Vec<OpUpdate> {
        let done = match self.op.as_ref() {
            None => return vec![],
            Some(Op::Ready(_)) => true,
            Some(Op::Thread(rx)) => match rx.try_recv() {
                Ok(r) => {
                    self.op = Some(Op::Ready(r));
                    true
                }
                Err(mpsc::TryRecvError::Empty) => false,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.op = Some(Op::Ready(OpResult::Read(Err(DriveError::Io("leitura interrompida".into())))));
                    true
                }
            },
        };
        match (done, self.op.take()) {
            (true, Some(Op::Ready(r))) => vec![OpUpdate::Done(r)],
            (_, op) => {
                self.op = op;
                vec![]
            }
        }
    }

    fn watch(&mut self, drive: &str) {
        if self.watched.as_deref() != Some(drive) {
            self.watched = Some(drive.to_string());
            self.seen = None; // o próximo ciclo só reobserva, sem gerar evento
        }
    }

    fn poll_events(&mut self) -> Vec<DriveEvent> {
        if self.last_poll.is_none_or(|t| t.elapsed() >= POLL) {
            self.last_poll = Some(Instant::now());
            let now = self.sample();
            // primeira observação é o estado inicial: disco já presente não dispara nada (ADR-0003)
            if let Some(prev) = &self.seen {
                for d in &now.drives {
                    if !prev.drives.contains(d) {
                        self.events.push(DriveEvent::DriveAdded);
                    }
                }
                for d in &prev.drives {
                    if !now.drives.contains(d) {
                        self.events.push(DriveEvent::DriveRemoved);
                    }
                }
                let watched_gone = self.watched.as_ref().is_some_and(|w| !now.drives.contains(w));
                if !watched_gone {
                    match (prev.media, now.media) {
                        (false, true) => self.events.push(DriveEvent::MediaArrived),
                        (true, false) => self.events.push(DriveEvent::MediaRemoved),
                        _ => {}
                    }
                }
            }
            self.seen = Some(now);
        }
        std::mem::take(&mut self.events)
    }
}

// ---------- Win32 ----------

#[cfg(windows)]
mod sys {
    use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW};
    use windows_sys::Win32::System::Diagnostics::Debug::{SetErrorMode, SEM_FAILCRITICALERRORS};

    const DRIVE_CDROM: u32 = 5;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    pub fn optical_drives() -> Vec<String> {
        // SAFETY: chamadas simples sem ponteiros de saída; `SetErrorMode` evita o diálogo "insira um disco".
        unsafe {
            SetErrorMode(SEM_FAILCRITICALERRORS);
            let mask = GetLogicalDrives();
            (0..26u32)
                .filter(|i| mask & (1 << i) != 0)
                .map(|i| format!("{}:", (b'A' + i as u8) as char))
                .filter(|d| GetDriveTypeW(wide(&format!("{d}\\")).as_ptr()) == DRIVE_CDROM)
                .collect()
        }
    }

    /// `Some(rótulo)` se o volume está pronto (há mídia legível); `None` sem mídia.
    pub fn volume_label(drive: &str) -> Option<String> {
        let mut name = [0u16; 261];
        // SAFETY: `name` tem 261 unidades e o tamanho é informado; os demais ponteiros são nulos e opcionais.
        let ok = unsafe {
            SetErrorMode(SEM_FAILCRITICALERRORS);
            GetVolumeInformationW(wide(&format!("{drive}\\")).as_ptr(), name.as_mut_ptr(), name.len() as u32, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), 0)
        };
        (ok != 0).then(|| String::from_utf16_lossy(&name[..name.iter().position(|&c| c == 0).unwrap_or(name.len())]))
    }
}

#[cfg(windows)]
use sys::{optical_drives, volume_label};

#[cfg(not(windows))]
fn optical_drives() -> Vec<String> {
    Vec::new() // Linux é a fase 2 (udisks2)
}

#[cfg(not(windows))]
fn volume_label(_: &str) -> Option<String> {
    None
}

fn volume_ready(drive: &str) -> bool {
    volume_label(drive).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("insert-disc-windrive-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn root_with_game_ini_is_data_and_carries_the_ini_and_label() {
        let d = dir("ini");
        fs::write(d.join("game.ini"), b"[disc]\r\nid = x\r\n").unwrap(); // caixa qualquer
        fs::write(d.join("OUTRO.TXT"), b"x").unwrap();
        let m = read_root(&d, Some("HADES".into()));
        assert_eq!((m.kind, m.physical, m.label.as_deref()), (MediaKind::Data, Physical::Unknown, Some("HADES")));
        assert_eq!(m.game_ini.as_deref(), Some(&b"[disc]\r\nid = x\r\n"[..]));
    }

    #[test]
    fn oversize_ini_is_capped_and_other_roots_are_classified() {
        let d = dir("big");
        fs::write(d.join("GAME.INI"), vec![b'a'; MAX_INI_BYTES * 3]).unwrap();
        assert_eq!(read_root(&d, None).game_ini.unwrap().len(), MAX_INI_BYTES + 1);

        let audio = dir("audio");
        fs::write(audio.join("Track01.cda"), b"").unwrap();
        fs::write(audio.join("TRACK02.CDA"), b"").unwrap();
        assert_eq!(read_root(&audio, None).kind, MediaKind::Audio);

        assert_eq!(read_root(&dir("empty"), None).kind, MediaKind::Blank);
        assert_eq!(read_root(Path::new("Z:\\nao-existe\\x"), None).kind, MediaKind::Unreadable);
        let plain = dir("plain");
        fs::write(plain.join("LEIAME.TXT"), b"x").unwrap();
        let m = read_root(&plain, None);
        assert!((m.kind, m.game_ini.is_none()) == (MediaKind::Data, true));
    }

    #[test]
    fn writing_and_tray_are_unsupported_until_there_is_hardware() {
        let mut d = WindowsDrive::new();
        assert_eq!(d.open_tray("E:"), Err(DriveError::Unsupported));
        assert_eq!(d.burn("E:", &DiscImageSpec { label: "X".into(), game_ini: String::new() }, &mut |_| {}), Err(DriveError::Unsupported));
        d.start_op("E:", DriveOp::Erase { quick: true }, 0);
        assert_eq!(d.poll_ops(0), vec![OpUpdate::Done(OpResult::Erase(Err(DriveError::Unsupported)))]);
        assert!(d.poll_ops(1).is_empty());
    }
}
