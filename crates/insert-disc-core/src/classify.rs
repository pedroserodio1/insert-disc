//! Classificação da mídia (docs/UX-STATES.md, "Classificação de mídia").

use crate::catalog::{Catalog, GameId};
use crate::drive::{MediaInfo, MediaKind};
use crate::gameini::{self, IniError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaClass {
    /// `id` associado ao jogo esperado.
    Match,
    /// Sem jogo esperado: `id` associado a este jogo.
    Known { game: GameId },
    /// Com jogo esperado: `id` associado a outro jogo.
    OtherGame { other: GameId },
    Unknown { ini_name: Option<String> },
    LegacyGameIni,
    NoGameIni,
    InvalidGameIni,
    Blank,
    Audio,
    ReadError,
}

impl MediaClass {
    /// Chave usada nos textos (`reject.<CHAVE>`, docs/FRONTEND-DESIGN.md §9).
    pub fn key(&self) -> &'static str {
        match self {
            MediaClass::Match => "MATCH",
            MediaClass::Known { .. } => "KNOWN",
            MediaClass::OtherGame { .. } => "OTHER_GAME",
            MediaClass::Unknown { .. } => "UNKNOWN",
            MediaClass::LegacyGameIni => "LEGACY_GAME_INI",
            MediaClass::NoGameIni => "NO_GAME_INI",
            MediaClass::InvalidGameIni => "INVALID_GAME_INI",
            MediaClass::Blank => "BLANK",
            MediaClass::Audio => "AUDIO",
            MediaClass::ReadError => "READ_ERROR",
        }
    }
}

pub fn classify(media: &MediaInfo, catalog: &Catalog, expected: Option<GameId>) -> MediaClass {
    match media.kind {
        MediaKind::Blank => MediaClass::Blank,
        MediaKind::Unreadable => MediaClass::ReadError,
        MediaKind::Audio => MediaClass::Audio,
        MediaKind::Mixed if media.game_ini.is_none() => MediaClass::Audio,
        MediaKind::Data | MediaKind::Mixed => match media.game_ini.as_deref().map(gameini::parse) {
            None => MediaClass::NoGameIni,
            Some(Err(IniError::Legacy)) => MediaClass::LegacyGameIni,
            Some(Err(_)) => MediaClass::InvalidGameIni,
            Some(Ok(ini)) => match catalog.game_of_disc(ini.id).map(|g| g.game_id) {
                None => MediaClass::Unknown { ini_name: ini.name },
                Some(g) => match expected {
                    Some(e) if e == g => MediaClass::Match,
                    Some(_) => MediaClass::OtherGame { other: g },
                    None => MediaClass::Known { game: g },
                },
            },
        },
    }
}
