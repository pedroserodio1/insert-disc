//! Catálogo local: única fonte do que é executado (ADR-0006). JSON com versão de esquema (Q3).
//! Escrita atômica com backups rotativos; arquivo ilegível nunca é sobrescrito.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type GameId = Uuid;
pub type DiscId = Uuid;

pub const CATALOG_VERSION: u32 = 1;
const BACKUPS: usize = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameKind {
    Steam { app_id: u32 },
    Custom {
        executable: PathBuf,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        working_dir: Option<PathBuf>,
        #[serde(default)]
        requires_elevation: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverSource {
    SteamCache,
    UserFile,
    Online,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiscMedia {
    CdR,
    CdRw,
    Iso,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscOrigin {
    Burned,
    Adopted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Disc {
    pub disc_id: DiscId,
    pub label: String,
    pub media: DiscMedia,
    /// Milissegundos desde a época Unix.
    pub created_at: u64,
    pub origin: DiscOrigin,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub game_id: GameId,
    pub name: String,
    #[serde(flatten)]
    pub kind: GameKind,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub cover_source: Option<CoverSource>,
    #[serde(default)]
    pub spine_color: Option<String>,
    #[serde(default)]
    pub discs: Vec<Disc>,
}

impl Game {
    pub fn new(name: impl Into<String>, kind: GameKind) -> Self {
        Game {
            game_id: Uuid::new_v4(),
            name: name.into(),
            kind,
            cover: None,
            cover_source: None,
            spine_color: None,
            discs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnDiscInsert {
    #[default]
    Focus,
    Launch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowMode {
    #[default]
    Windowed,
    Fullscreen,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub drive: Option<String>,
    pub on_disc_insert: OnDiscInsert,
    pub loading_min_ms: u32,
    /// `pt-BR`, `en` ou `None` (segue o sistema).
    pub locale: Option<String>,
    pub window_mode: WindowMode,
    pub covers_online_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            drive: None,
            on_disc_insert: OnDiscInsert::Focus,
            loading_min_ms: 3000,
            locale: None,
            window_mode: WindowMode::Windowed,
            covers_online_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    pub version: u32,
    #[serde(default)]
    pub games: Vec<Game>,
    #[serde(default)]
    pub settings: Settings,
}

impl Default for Catalog {
    fn default() -> Self {
        Catalog { version: CATALOG_VERSION, games: Vec::new(), settings: Settings::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    InvalidGame(&'static str),
    DuplicateGame,
    DuplicateDisc,
    NoSuchGame,
    NoSuchDisc,
}

#[derive(Debug)]
pub enum LoadError {
    /// Arquivo existe mas não é utilizável. **Não foi tocado.**
    Corrupt(String),
    /// Escrito por uma versão mais nova do app.
    TooNew(u32),
}

fn is_batch(p: &Path) -> bool {
    p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("bat") || e.eq_ignore_ascii_case("cmd"))
}

/// Regras de validação de um jogo (SECURITY R3, Q13).
pub fn validate_game(g: &Game) -> Result<(), CatalogError> {
    if g.name.trim().is_empty() {
        return Err(CatalogError::InvalidGame("nome vazio"));
    }
    match &g.kind {
        GameKind::Steam { app_id } if *app_id == 0 => Err(CatalogError::InvalidGame("app_id inválido")),
        GameKind::Steam { .. } => Ok(()),
        GameKind::Custom { executable, args, working_dir, .. } => {
            if !executable.is_absolute() {
                return Err(CatalogError::InvalidGame("executável precisa ser caminho absoluto"));
            }
            if working_dir.as_ref().is_some_and(|w| !w.is_absolute()) {
                return Err(CatalogError::InvalidGame("diretório de trabalho precisa ser absoluto"));
            }
            if args.iter().any(|a| a.contains('\0')) {
                return Err(CatalogError::InvalidGame("argumento inválido"));
            }
            if is_batch(executable) && !args.is_empty() {
                return Err(CatalogError::InvalidGame(".bat/.cmd não aceitam argumentos"));
            }
            Ok(())
        }
    }
}

impl Catalog {
    pub fn game(&self, id: GameId) -> Option<&Game> {
        self.games.iter().find(|g| g.game_id == id)
    }

    pub fn game_of_disc(&self, disc: DiscId) -> Option<&Game> {
        self.games.iter().find(|g| g.discs.iter().any(|d| d.disc_id == disc))
    }

    pub fn add_game(&mut self, g: Game) -> Result<GameId, CatalogError> {
        validate_game(&g)?;
        if self.game(g.game_id).is_some() {
            return Err(CatalogError::DuplicateGame);
        }
        if g.discs.iter().any(|d| self.game_of_disc(d.disc_id).is_some()) {
            return Err(CatalogError::DuplicateDisc);
        }
        let id = g.game_id;
        self.games.push(g);
        Ok(id)
    }

    /// Invariante 1: `disc_id` único no catálogo inteiro.
    pub fn add_disc(&mut self, game: GameId, disc: Disc) -> Result<(), CatalogError> {
        if self.game_of_disc(disc.disc_id).is_some() {
            return Err(CatalogError::DuplicateDisc);
        }
        let g = self.games.iter_mut().find(|g| g.game_id == game).ok_or(CatalogError::NoSuchGame)?;
        g.discs.push(disc);
        Ok(())
    }

    pub fn remove_disc(&mut self, disc: DiscId) -> bool {
        self.games.iter_mut().any(|g| {
            let n = g.discs.len();
            g.discs.retain(|d| d.disc_id != disc);
            g.discs.len() != n
        })
    }

    /// Remove o jogo e as associações de disco (invariante 5).
    pub fn remove_game(&mut self, id: GameId) -> bool {
        let n = self.games.len();
        self.games.retain(|g| g.game_id != id);
        self.games.len() != n
    }

    pub fn rename_game(&mut self, id: GameId, name: &str) -> Result<(), CatalogError> {
        if name.trim().is_empty() {
            return Err(CatalogError::InvalidGame("nome vazio"));
        }
        let g = self.games.iter_mut().find(|g| g.game_id == id).ok_or(CatalogError::NoSuchGame)?;
        g.name = name.trim().to_string();
        Ok(())
    }

    /// Edita nome e tipo (executável, argumentos etc.) com a mesma validação da criação.
    pub fn update_game(&mut self, id: GameId, name: &str, kind: GameKind) -> Result<(), CatalogError> {
        let g = self.games.iter_mut().find(|g| g.game_id == id).ok_or(CatalogError::NoSuchGame)?;
        let candidate = Game { name: name.trim().to_string(), kind, ..g.clone() };
        validate_game(&candidate)?;
        *g = candidate;
        Ok(())
    }

    /// Ordem de exibição (Q21): alfabética, sem diferenciar maiúsculas.
    pub fn sorted_games(&self) -> Vec<&Game> {
        let mut v: Vec<&Game> = self.games.iter().collect();
        v.sort_by_key(|g| g.name.to_lowercase());
        v
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("catálogo sempre serializa")
    }

    pub fn from_json(s: &str) -> Result<Catalog, LoadError> {
        let c: Catalog = serde_json::from_str(s).map_err(|e| LoadError::Corrupt(e.to_string()))?;
        if c.version > CATALOG_VERSION {
            return Err(LoadError::TooNew(c.version));
        }
        let mut games = HashSet::new();
        let mut discs = HashSet::new();
        for g in &c.games {
            if !games.insert(g.game_id) {
                return Err(LoadError::Corrupt("game_id duplicado".into()));
            }
            validate_game(g).map_err(|e| LoadError::Corrupt(format!("jogo inválido: {e:?}")))?;
            for d in &g.discs {
                if !discs.insert(d.disc_id) {
                    return Err(LoadError::Corrupt("disc_id duplicado".into()));
                }
            }
        }
        Ok(c)
    }

    /// Arquivo ausente = estante vazia (primeira execução).
    pub fn load(path: &Path) -> Result<Catalog, LoadError> {
        match fs::read_to_string(path) {
            Ok(s) => Catalog::from_json(&s),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Catalog::default()),
            Err(e) => Err(LoadError::Corrupt(e.to_string())),
        }
    }

    /// Escrita atômica (arquivo temporário + troca) com backups rotativos.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = with_suffix(path, ".tmp");
        fs::write(&tmp, self.to_json())?;
        if path.exists() {
            for n in (1..BACKUPS).rev() {
                let from = with_suffix(path, &format!(".bak{n}"));
                if from.exists() {
                    fs::rename(&from, with_suffix(path, &format!(".bak{}", n + 1)))?;
                }
            }
            fs::copy(path, with_suffix(path, ".bak1"))?;
        }
        fs::rename(&tmp, path)
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// Backups legíveis, do mais novo (índice 0) ao mais antigo.
pub fn list_backups(path: &Path) -> Vec<PathBuf> {
    (1..=BACKUPS)
        .map(|n| with_suffix(path, &format!(".bak{n}")))
        .filter(|p| fs::read_to_string(p).is_ok_and(|s| Catalog::from_json(&s).is_ok()))
        .collect()
}

/// Preserva o arquivo corrompido ao lado (nunca apaga).
fn preserve_corrupt(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let mut n = 1;
    loop {
        let dest = with_suffix(path, &format!(".corrupt-{n}"));
        if !dest.exists() {
            return fs::rename(path, dest);
        }
        n += 1;
    }
}

pub fn start_empty(path: &Path) -> io::Result<Catalog> {
    preserve_corrupt(path)?;
    Ok(Catalog::default())
}

pub fn restore_backup(path: &Path, index: usize) -> io::Result<Catalog> {
    let backup = list_backups(path).into_iter().nth(index).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "backup"))?;
    let cat = Catalog::from_json(&fs::read_to_string(&backup)?).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "backup"))?;
    preserve_corrupt(path)?;
    cat.save(path)?;
    Ok(cat)
}

/// Importar estante (Q19, opção a): substitui, com backup automático do arquivo atual.
pub fn import_replace(path: &Path, json: &str) -> Result<Catalog, LoadError> {
    let cat = Catalog::from_json(json)?;
    cat.save(path).map_err(|e| LoadError::Corrupt(e.to_string()))?;
    Ok(cat)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn abs(p: &str) -> PathBuf {
        if cfg!(windows) { PathBuf::from(format!("C:\\{p}")) } else { PathBuf::from(format!("/{p}")) }
    }

    fn custom(name: &str, exe: &str, args: &[&str]) -> Game {
        Game::new(name, GameKind::Custom {
            executable: abs(exe),
            args: args.iter().map(|s| s.to_string()).collect(),
            working_dir: None,
            requires_elevation: false,
        })
    }

    fn disc() -> Disc {
        Disc { disc_id: Uuid::new_v4(), label: "X".into(), media: DiscMedia::CdR, created_at: 1, origin: DiscOrigin::Burned, note: None }
    }

    #[test]
    fn validation_rules() {
        let mut c = Catalog::default();
        assert!(c.add_game(Game::new("Ok", GameKind::Steam { app_id: 620 })).is_ok());
        assert!(c.add_game(Game::new("Zero", GameKind::Steam { app_id: 0 })).is_err());
        assert!(c.add_game(Game::new("  ", GameKind::Steam { app_id: 1 })).is_err());
        assert!(c.add_game(Game::new("Rel", GameKind::Custom { executable: "rel.exe".into(), args: vec![], working_dir: None, requires_elevation: false })).is_err());
        assert!(c.add_game(custom("Bat", "g\\run.bat", &["x"])).is_err());
        assert!(c.add_game(custom("BatNoArgs", "g\\run.cmd", &[])).is_ok());
        assert!(c.add_game(custom("Nul", "g\\a.exe", &["a\0b"])).is_err());
    }

    #[test]
    fn disc_ids_are_unique_and_removal_orphans_discs() {
        let mut c = Catalog::default();
        let a = c.add_game(Game::new("A", GameKind::Steam { app_id: 1 })).unwrap();
        let b = c.add_game(Game::new("B", GameKind::Steam { app_id: 2 })).unwrap();
        let d = disc();
        c.add_disc(a, d.clone()).unwrap();
        assert_eq!(c.add_disc(b, d.clone()), Err(CatalogError::DuplicateDisc));
        assert_eq!(c.game_of_disc(d.disc_id).unwrap().game_id, a);
        assert!(c.remove_game(a));
        assert!(c.game_of_disc(d.disc_id).is_none());
        assert_eq!(c.add_disc(a, disc()), Err(CatalogError::NoSuchGame));
    }

    #[test]
    fn json_roundtrip_and_corruption_checks() {
        let mut c = Catalog::default();
        let a = c.add_game(custom("Emu", "e\\emu.exe", &["--rom", "C:\\r\\a b.sfc"])).unwrap();
        c.add_disc(a, disc()).unwrap();
        assert_eq!(Catalog::from_json(&c.to_json()).unwrap(), c);
        assert!(matches!(Catalog::from_json("{not json"), Err(LoadError::Corrupt(_))));
        assert!(matches!(Catalog::from_json(r#"{"version":99}"#), Err(LoadError::TooNew(99))));
        let mut dup = c.clone();
        let g2 = dup.games[0].clone();
        dup.games.push(g2);
        assert!(matches!(Catalog::from_json(&dup.to_json()), Err(LoadError::Corrupt(_))));
    }

    #[test]
    fn save_is_atomic_with_rotating_backups_and_corrupt_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("catalog.json");
        assert_eq!(Catalog::load(&path).unwrap(), Catalog::default()); // ausente = vazio

        let mut c = Catalog::default();
        for i in 1..=5u32 {
            c.add_game(Game::new(format!("G{i}"), GameKind::Steam { app_id: i })).unwrap();
            c.save(&path).unwrap();
        }
        assert_eq!(Catalog::load(&path).unwrap().games.len(), 5);
        assert_eq!(list_backups(&path).len(), 3);
        assert_eq!(Catalog::from_json(&fs::read_to_string(&list_backups(&path)[0]).unwrap()).unwrap().games.len(), 4);
        assert!(!with_suffix(&path, ".tmp").exists());

        fs::write(&path, "lixo").unwrap();
        assert!(matches!(Catalog::load(&path), Err(LoadError::Corrupt(_))));
        assert_eq!(fs::read_to_string(&path).unwrap(), "lixo"); // load não toca no arquivo

        let restored = restore_backup(&path, 0).unwrap();
        assert_eq!(restored.games.len(), 4);
        assert_eq!(fs::read_to_string(with_suffix(&path, ".corrupt-1")).unwrap(), "lixo");
        assert_eq!(Catalog::load(&path).unwrap().games.len(), 4);

        fs::write(&path, "lixo2").unwrap();
        let empty = start_empty(&path).unwrap();
        assert!(empty.games.is_empty());
        assert_eq!(fs::read_to_string(with_suffix(&path, ".corrupt-1")).unwrap(), "lixo");
        assert_eq!(fs::read_to_string(with_suffix(&path, ".corrupt-2")).unwrap(), "lixo2");
    }

    #[test]
    fn import_replaces_and_backs_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.json");
        let mut a = Catalog::default();
        a.add_game(Game::new("Antes", GameKind::Steam { app_id: 1 })).unwrap();
        a.save(&path).unwrap();
        let mut b = Catalog::default();
        b.add_game(Game::new("Depois", GameKind::Steam { app_id: 2 })).unwrap();
        import_replace(&path, &b.to_json()).unwrap();
        assert_eq!(Catalog::load(&path).unwrap().games[0].name, "Depois");
        assert_eq!(list_backups(&path).len(), 1);
        assert!(import_replace(&path, "x").is_err());
        assert_eq!(Catalog::load(&path).unwrap().games[0].name, "Depois");
    }

    #[test]
    fn sorted_case_insensitive() {
        let mut c = Catalog::default();
        for n in ["zelda", "Alpha", "beta"] {
            c.add_game(Game::new(n, GameKind::Steam { app_id: 1 })).unwrap();
        }
        let names: Vec<_> = c.sorted_games().iter().map(|g| g.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "beta", "zelda"]);
    }
}
