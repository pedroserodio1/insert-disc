//! O drive que o app usa: o falso (ISO, desenvolvimento e demonstração) ou o do Windows.
//! Enum em vez de `dyn` para o `Host` continuar concreto e o painel de desenvolvimento poder
//! alcançar o drive falso.

use insert_disc_core::drive::*;
use insert_disc_core::fake::FakeIsoDrive;

use crate::windrive::WindowsDrive;

pub enum AnyDrive {
    Fake(FakeIsoDrive),
    Windows(WindowsDrive),
}

impl AnyDrive {
    pub fn fake_mut(&mut self) -> Option<&mut FakeIsoDrive> {
        match self {
            AnyDrive::Fake(d) => Some(d),
            AnyDrive::Windows(_) => None,
        }
    }

    pub fn fake(&self) -> Option<&FakeIsoDrive> {
        match self {
            AnyDrive::Fake(d) => Some(d),
            AnyDrive::Windows(_) => None,
        }
    }
}

macro_rules! delegate {
    ($self:ident, $d:ident => $e:expr) => {
        match $self {
            AnyDrive::Fake($d) => $e,
            AnyDrive::Windows($d) => $e,
        }
    };
}

impl DriveBackend for AnyDrive {
    fn kind(&self) -> BackendKind {
        delegate!(self, d => d.kind())
    }
    fn list_drives(&mut self) -> Vec<DriveInfo> {
        delegate!(self, d => d.list_drives())
    }
    fn capabilities(&mut self, drive: &str) -> Capabilities {
        delegate!(self, d => d.capabilities(drive))
    }
    fn media_present(&mut self, drive: &str) -> bool {
        delegate!(self, d => d.media_present(drive))
    }
    fn open_tray(&mut self, drive: &str) -> Result<(), DriveError> {
        delegate!(self, d => d.open_tray(drive))
    }
    fn close_tray(&mut self, drive: &str) -> Result<(), DriveError> {
        delegate!(self, d => d.close_tray(drive))
    }
    fn eject(&mut self, drive: &str) -> Result<(), DriveError> {
        delegate!(self, d => d.eject(drive))
    }
    fn read_media(&mut self, drive: &str) -> Result<MediaInfo, DriveError> {
        delegate!(self, d => d.read_media(drive))
    }
    fn burn(&mut self, drive: &str, spec: &DiscImageSpec, progress: &mut dyn FnMut(u8)) -> Result<(), DriveError> {
        delegate!(self, d => d.burn(drive, spec, progress))
    }
    fn erase(&mut self, drive: &str, quick: bool, progress: &mut dyn FnMut(u8)) -> Result<(), DriveError> {
        delegate!(self, d => d.erase(drive, quick, progress))
    }
    fn start_op(&mut self, drive: &str, op: DriveOp, now: u64) {
        delegate!(self, d => d.start_op(drive, op, now))
    }
    fn poll_ops(&mut self, now: u64) -> Vec<OpUpdate> {
        delegate!(self, d => d.poll_ops(now))
    }
    fn watch(&mut self, drive: &str) {
        delegate!(self, d => d.watch(drive))
    }
    fn poll_events(&mut self) -> Vec<DriveEvent> {
        delegate!(self, d => d.poll_events())
    }
}
