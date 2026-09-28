//! Política de lançamento: catálogo -> `LaunchRequest`. Não executa nada (isso é da
//! `SystemIntegration`). A URI da Steam só é montada a partir de um inteiro (SECURITY R3).

use std::path::PathBuf;

use crate::catalog::{Game, GameKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchRequest {
    Uri(String),
    Exec { executable: PathBuf, args: Vec<String>, working_dir: Option<PathBuf>, elevated: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchError {
    NotFound,
    SteamMissing,
    ElevationDenied,
    Generic(String),
}

impl LaunchError {
    /// Chave do texto `launch.error.<chave>`.
    pub fn key(&self) -> &'static str {
        match self {
            LaunchError::NotFound => "not_found",
            LaunchError::SteamMissing => "steam_missing",
            LaunchError::ElevationDenied => "elevation_denied",
            LaunchError::Generic(_) => "generic",
        }
    }
}

pub trait Launcher {
    fn launch(&mut self, req: &LaunchRequest) -> Result<(), LaunchError>;
}

pub fn request_for(game: &Game) -> LaunchRequest {
    match &game.kind {
        GameKind::Steam { app_id } => LaunchRequest::Uri(format!("steam://run/{app_id}")),
        GameKind::Custom { executable, args, working_dir, requires_elevation } => LaunchRequest::Exec {
            executable: executable.clone(),
            args: args.clone(),
            working_dir: working_dir.clone().or_else(|| executable.parent().map(|p| p.to_path_buf())),
            elevated: *requires_elevation,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_uri_is_built_from_a_number_only() {
        let g = Game::new("Portal", GameKind::Steam { app_id: 400 });
        assert_eq!(request_for(&g), LaunchRequest::Uri("steam://run/400".into()));
    }

    #[test]
    fn custom_keeps_args_as_a_list_and_defaults_the_working_dir() {
        let exe = if cfg!(windows) { PathBuf::from("C:\\emu\\emu.exe") } else { PathBuf::from("/emu/emu.exe") };
        let g = Game::new("X", GameKind::Custom {
            executable: exe.clone(),
            args: vec!["--rom".into(), "a b; calc.exe".into()],
            working_dir: None,
            requires_elevation: false,
        });
        match request_for(&g) {
            LaunchRequest::Exec { args, working_dir, .. } => {
                assert_eq!(args, ["--rom", "a b; calc.exe"]); // um elemento só, sem interpretação
                assert_eq!(working_dir, exe.parent().map(|p| p.to_path_buf()));
            }
            other => panic!("{other:?}"),
        }
    }
}
