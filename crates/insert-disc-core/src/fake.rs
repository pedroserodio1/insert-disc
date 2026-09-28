//! `FakeIsoDrive` (docs/DRIVE-LAYER.md): simula inserção, ejeção, mídia virgem, CD de áudio,
//! falha de leitura e gravação, sem hardware e sem montar nada no SO.
//! "Gravar" gera um `.iso` novo, reinserível.

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::drive::*;
use crate::gameini::MAX_INI_BYTES;
use crate::iso;

const DRIVE: &str = "fake:0";

#[derive(Debug, Clone)]
enum Slot {
    /// ISO lida diretamente; `physical` diz se é CD-ROM prensado ou regravável.
    Iso { path: PathBuf, physical: Physical },
    Blank(Physical),
    Audio,
    ReadError,
    /// CD-R cuja gravação falhou: perdido.
    LostCdr,
    /// CD-RW cuja gravação falhou: precisa apagar.
    DirtyCdrw,
}

pub struct FakeIsoDrive {
    caps: Capabilities,
    present: bool,
    slot: Option<Slot>,
    events: Vec<DriveEvent>,
    out_dir: PathBuf,
    fail_burn: bool,
    fail_erase: bool,
    fail_open_tray: bool,
    burn_count: u32,
}

impl FakeIsoDrive {
    pub fn new(out_dir: impl Into<PathBuf>) -> Self {
        FakeIsoDrive {
            caps: Capabilities::ALL,
            present: true,
            slot: None,
            events: Vec::new(),
            out_dir: out_dir.into(),
            fail_burn: false,
            fail_erase: false,
            fail_open_tray: false,
            burn_count: 0,
        }
    }

    pub fn capabilities_now(&self) -> Capabilities {
        self.caps
    }
    pub fn set_capabilities(&mut self, caps: Capabilities) {
        self.caps = caps;
    }
    pub fn set_fail_burn(&mut self, v: bool) {
        self.fail_burn = v;
    }
    pub fn set_fail_erase(&mut self, v: bool) {
        self.fail_erase = v;
    }
    pub fn set_fail_open_tray(&mut self, v: bool) {
        self.fail_open_tray = v;
    }

    fn put(&mut self, slot: Slot) {
        self.slot = Some(slot);
        self.events.push(DriveEvent::MediaArrived);
    }

    /// CD-ROM prensado (somente leitura).
    pub fn insert_iso(&mut self, path: impl Into<PathBuf>) {
        self.put(Slot::Iso { path: path.into(), physical: Physical::CdRom });
    }
    /// CD-RW já gravado (regravável).
    pub fn insert_rewritable_iso(&mut self, path: impl Into<PathBuf>) {
        self.put(Slot::Iso { path: path.into(), physical: Physical::CdRw });
    }
    /// CD-R já gravado.
    pub fn insert_cdr_iso(&mut self, path: impl Into<PathBuf>) {
        self.put(Slot::Iso { path: path.into(), physical: Physical::CdR });
    }
    pub fn insert_blank_cdr(&mut self) {
        self.put(Slot::Blank(Physical::CdR));
    }
    pub fn insert_blank_cdrw(&mut self) {
        self.put(Slot::Blank(Physical::CdRw));
    }
    pub fn insert_audio(&mut self) {
        self.put(Slot::Audio);
    }
    pub fn insert_unreadable(&mut self) {
        self.put(Slot::ReadError);
    }
    /// Remoção pelo usuário (puxar o disco).
    pub fn remove_media(&mut self) {
        if self.slot.take().is_some() {
            self.events.push(DriveEvent::MediaRemoved);
        }
    }
    /// Evento repetido (cenário C17/C18): não muda o estado da mídia.
    pub fn inject_duplicate_arrival(&mut self) {
        self.events.push(DriveEvent::MediaArrived);
    }
    pub fn disconnect(&mut self) {
        self.present = false;
        self.slot = None;
        self.events.push(DriveEvent::DriveRemoved);
    }
    pub fn reconnect(&mut self) {
        self.present = true;
        self.events.push(DriveEvent::DriveAdded);
    }
    pub fn has_media(&self) -> bool {
        self.slot.is_some()
    }
    /// Último `.iso` gravado, se houver.
    pub fn burned_path(&self) -> Option<PathBuf> {
        match &self.slot {
            Some(Slot::Iso { path, .. }) if path.starts_with(&self.out_dir) => Some(path.clone()),
            _ => None,
        }
    }
    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }
}

impl DriveBackend for FakeIsoDrive {
    fn kind(&self) -> BackendKind {
        BackendKind::Fake
    }

    fn list_drives(&mut self) -> Vec<DriveInfo> {
        if self.present { vec![DriveInfo { id: DRIVE.into(), name: "Drive falso (ISO)".into() }] } else { vec![] }
    }

    fn capabilities(&mut self, _: &str) -> Capabilities {
        self.caps
    }

    fn media_present(&mut self, _: &str) -> bool {
        self.present && self.slot.is_some()
    }

    fn open_tray(&mut self, _: &str) -> Result<(), DriveError> {
        if self.caps.tray_open != Tri::Yes {
            return Err(DriveError::Unsupported);
        }
        if self.fail_open_tray {
            return Err(DriveError::Io("gaveta travada".into()));
        }
        Ok(())
    }

    fn close_tray(&mut self, _: &str) -> Result<(), DriveError> {
        if self.caps.tray_close != Tri::Yes { Err(DriveError::Unsupported) } else { Ok(()) }
    }

    fn eject(&mut self, _: &str) -> Result<(), DriveError> {
        if self.caps.eject != Tri::Yes {
            return Err(DriveError::Unsupported);
        }
        self.remove_media();
        Ok(())
    }

    fn read_media(&mut self, _: &str) -> Result<MediaInfo, DriveError> {
        if !self.present {
            return Err(DriveError::Removed);
        }
        let media = |kind, physical, label, game_ini| MediaInfo { kind, physical, label, game_ini };
        match self.slot.as_ref().ok_or(DriveError::NoMedia)? {
            Slot::Blank(p) => Ok(media(MediaKind::Blank, *p, None, None)),
            Slot::Audio => Ok(media(MediaKind::Audio, Physical::CdR, None, None)),
            Slot::ReadError => Ok(MediaInfo::unreadable()),
            Slot::LostCdr => Ok(MediaInfo { physical: Physical::CdR, ..MediaInfo::unreadable() }),
            Slot::DirtyCdrw => Ok(MediaInfo { physical: Physical::CdRw, ..MediaInfo::unreadable() }),
            Slot::Iso { path, physical } => {
                let read = File::open(path)
                    .map_err(iso::IsoError::from)
                    .and_then(|mut f| iso::read_label_and_file(&mut f, "GAME.INI", MAX_INI_BYTES + 1));
                Ok(match read {
                    Ok((label, ini)) => media(MediaKind::Data, *physical, Some(label), ini),
                    Err(_) => MediaInfo { physical: *physical, ..MediaInfo::unreadable() },
                })
            }
        }
    }

    fn burn(&mut self, _: &str, spec: &DiscImageSpec, progress: &mut dyn FnMut(u8)) -> Result<(), DriveError> {
        let physical = match self.slot.as_ref().ok_or(DriveError::NoMedia)? {
            Slot::Blank(p) => *p,
            _ => return Err(DriveError::Io("mídia não está virgem".into())),
        };
        let allowed = if physical == Physical::CdR { self.caps.write_cdr } else { self.caps.write_cdrw };
        if allowed == Tri::No {
            return Err(DriveError::Unsupported);
        }
        progress(0);
        if self.fail_burn {
            progress(40);
            self.slot = Some(if physical == Physical::CdR { Slot::LostCdr } else { Slot::DirtyCdrw });
            return Err(DriveError::Io("falha simulada no meio da gravação".into()));
        }
        fs::create_dir_all(&self.out_dir).map_err(|e| DriveError::Io(e.to_string()))?;
        self.burn_count += 1;
        let path = self.out_dir.join(format!("{}-{}.iso", spec.label, self.burn_count));
        let image = iso::build_iso(&spec.label, &[("GAME.INI", spec.game_ini.as_bytes())]);
        fs::write(&path, image).map_err(|e| DriveError::Io(e.to_string()))?;
        progress(100);
        self.slot = Some(Slot::Iso { path, physical });
        Ok(())
    }

    fn erase(&mut self, _: &str, _quick: bool, progress: &mut dyn FnMut(u8)) -> Result<(), DriveError> {
        match self.slot.as_ref().ok_or(DriveError::NoMedia)? {
            Slot::Iso { physical: Physical::CdRw, .. } | Slot::DirtyCdrw | Slot::Blank(Physical::CdRw) => {}
            _ => return Err(DriveError::Unsupported),
        }
        if self.caps.erase == Tri::No {
            return Err(DriveError::Unsupported);
        }
        progress(0);
        if self.fail_erase {
            return Err(DriveError::Io("falha simulada ao apagar".into()));
        }
        progress(100);
        self.slot = Some(Slot::Blank(Physical::CdRw));
        Ok(())
    }

    fn poll_events(&mut self) -> Vec<DriveEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameini;
    use uuid::Uuid;

    fn spec(id: Uuid) -> DiscImageSpec {
        DiscImageSpec { label: "CELESTE".into(), game_ini: gameini::write(id, "Celeste") }
    }

    #[test]
    fn burn_then_read_back_the_generated_iso() {
        let dir = tempfile::tempdir().unwrap();
        let mut d = FakeIsoDrive::new(dir.path());
        d.insert_blank_cdrw();
        assert_eq!(d.read_media(DRIVE).unwrap().kind, MediaKind::Blank);
        let id = Uuid::new_v4();
        let mut last = 0;
        d.burn(DRIVE, &spec(id), &mut |p| last = p).unwrap();
        assert_eq!(last, 100);
        let m = d.read_media(DRIVE).unwrap();
        assert_eq!((m.kind, m.physical, m.label.as_deref()), (MediaKind::Data, Physical::CdRw, Some("CELESTE")));
        assert_eq!(m.ini().unwrap().id, id);
        assert!(d.burned_path().unwrap().exists());
        // não dá para gravar de novo por cima
        assert!(d.burn(DRIVE, &spec(id), &mut |_| {}).is_err());
        // CD-RW apaga e volta a ser virgem
        d.erase(DRIVE, true, &mut |_| {}).unwrap();
        assert_eq!(d.read_media(DRIVE).unwrap().kind, MediaKind::Blank);
    }

    #[test]
    fn failed_burn_loses_cdr_and_dirties_cdrw() {
        let dir = tempfile::tempdir().unwrap();
        let mut d = FakeIsoDrive::new(dir.path());
        d.set_fail_burn(true);
        d.insert_blank_cdr();
        assert!(d.burn(DRIVE, &spec(Uuid::new_v4()), &mut |_| {}).is_err());
        assert_eq!(d.read_media(DRIVE).unwrap().kind, MediaKind::Unreadable);
        assert!(d.erase(DRIVE, true, &mut |_| {}).is_err()); // CD-R não apaga
        d.remove_media();
        d.insert_blank_cdrw();
        assert!(d.burn(DRIVE, &spec(Uuid::new_v4()), &mut |_| {}).is_err());
        d.erase(DRIVE, true, &mut |_| {}).unwrap(); // CD-RW se recupera
        assert_eq!(d.read_media(DRIVE).unwrap().kind, MediaKind::Blank);
    }

    #[test]
    fn events_and_capabilities() {
        let dir = tempfile::tempdir().unwrap();
        let mut d = FakeIsoDrive::new(dir.path());
        assert!(d.poll_events().is_empty());
        d.insert_audio();
        d.inject_duplicate_arrival();
        assert_eq!(d.poll_events(), [DriveEvent::MediaArrived, DriveEvent::MediaArrived]);
        d.eject(DRIVE).unwrap();
        assert_eq!(d.poll_events(), [DriveEvent::MediaRemoved]);
        d.set_capabilities(Capabilities { eject: Tri::No, tray_open: Tri::No, ..Capabilities::ALL });
        assert_eq!(d.open_tray(DRIVE), Err(DriveError::Unsupported));
        d.insert_audio();
        assert_eq!(d.eject(DRIVE), Err(DriveError::Unsupported));
        assert!(d.has_media());
        d.disconnect();
        assert!(d.list_drives().is_empty());
        assert_eq!(d.read_media(DRIVE), Err(DriveError::Removed));
    }

    #[test]
    fn corrupt_iso_reads_as_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("bad.iso");
        fs::write(&p, vec![7u8; 5000]).unwrap();
        let mut d = FakeIsoDrive::new(dir.path());
        d.insert_iso(&p);
        assert_eq!(d.read_media(DRIVE).unwrap().kind, MediaKind::Unreadable);
    }
}
