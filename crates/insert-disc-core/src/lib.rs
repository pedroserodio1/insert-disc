//! Núcleo portátil do Insert Disc. Nenhuma API de SO aqui (ADR-0010):
//! tudo que é de plataforma entra por `DriveBackend` e `Launcher`.

pub mod app;
pub mod catalog;
pub mod classify;
pub mod drive;
pub mod fake;
pub mod gameini;
pub mod iso;
pub mod launch;
pub mod snapshot;
