//! Estante de demonstração e discos de teste do drive falso (só desenvolvimento).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use insert_disc_core::catalog::*;
use insert_disc_core::fake::FakeIsoDrive;
use insert_disc_core::{gameini, iso};
use uuid::Uuid;

struct Entry {
    name: &'static str,
    app_id: Option<u32>,
    disc: bool,
    cover: bool,
    spine: &'static str,
}

const GAMES: &[Entry] = &[
    Entry { name: "Hollow Knight", app_id: Some(367520), disc: true, cover: true, spine: "#2E5A7A" },
    Entry { name: "Celeste", app_id: Some(504230), disc: true, cover: true, spine: "#7A2E4A" },
    Entry { name: "Hades", app_id: Some(1145360), disc: true, cover: true, spine: "#7A2E2E" },
    Entry { name: "Portal 2", app_id: Some(620), disc: true, cover: false, spine: "#4A4A6B" },
    Entry { name: "Stardew Valley", app_id: Some(413150), disc: true, cover: true, spine: "#3B6B3B" },
    Entry { name: "Undertale", app_id: Some(391540), disc: true, cover: false, spine: "#55366B" },
    Entry { name: "Cuphead", app_id: Some(268910), disc: true, cover: true, spine: "#6B4E1F" },
    Entry { name: "Dead Cells", app_id: Some(588650), disc: true, cover: false, spine: "#1F5F5F" },
    Entry { name: "Outer Wilds", app_id: Some(753640), disc: true, cover: true, spine: "#7A3B1F" },
    Entry { name: "Disco Elysium", app_id: Some(632470), disc: true, cover: false, spine: "#5A5A2E" },
    Entry { name: "Super Meat Boy", app_id: Some(40800), disc: true, cover: true, spine: "#7A2E2E" },
    Entry { name: "Katana ZERO", app_id: Some(460950), disc: true, cover: false, spine: "#2E4A6B" },
    Entry { name: "Super Metroid (SNES)", app_id: None, disc: true, cover: true, spine: "#4A2E6B" },
    Entry { name: "The Legend of Zelda: Breath of the Wild (Cemu)", app_id: None, disc: true, cover: false, spine: "#2E6B4A" },
    Entry { name: "Balatro", app_id: Some(2379780), disc: false, cover: true, spine: "#6B2E5A" },
    Entry { name: "Sem capa e sem disco", app_id: Some(999), disc: false, cover: false, spine: "#4A4A4A" },
];

pub struct Demo {
    dir: PathBuf,
    /// jogo -> ISO do disco dele
    game_iso: HashMap<Uuid, PathBuf>,
    extra: HashMap<&'static str, PathBuf>,
}

fn write_iso(dir: &Path, file: &str, label: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let p = dir.join(file);
    fs::write(&p, iso::build_iso(label, files)).expect("escrever ISO de demonstração");
    p
}

pub fn build(dir: &Path) -> (Catalog, Demo) {
    let isos = dir.join("isos");
    // apaga a pasta inteira: só aceita pastas de demonstração/teste (nunca os dados do usuário)
    assert!(dir.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("insert-disc-")), "pasta de demonstração inválida");
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(&isos).expect("criar pasta de demonstração");

    let mut catalog = Catalog::default();
    let mut game_iso = HashMap::new();
    for (i, e) in GAMES.iter().enumerate() {
        let kind = match e.app_id {
            Some(app_id) => GameKind::Steam { app_id },
            None => GameKind::Custom {
                executable: PathBuf::from(if cfg!(windows) { r"C:\Emulators\emu.exe" } else { "/opt/emu/emu" }),
                args: vec!["--rom".into(), format!("roms/{}.rom", e.name)],
                working_dir: None,
                requires_elevation: false,
            },
        };
        let mut g = Game::new(e.name, kind);
        g.spine_color = Some(e.spine.into());
        if e.cover {
            g.cover = Some(format!("/demo-cover/{}.svg", i % 8));
            g.cover_source = Some(CoverSource::UserFile);
        }
        if e.disc {
            let disc_id = Uuid::new_v4();
            let label = gameini::sanitize_label(e.name);
            let ini = gameini::write(disc_id, e.name);
            let path = write_iso(&isos, &format!("{label}.iso"), &label, &[("GAME.INI", ini.as_bytes())]);
            game_iso.insert(g.game_id, path);
            g.discs.push(Disc { disc_id, label, media: DiscMedia::Iso, created_at: 1_700_000_000_000, origin: DiscOrigin::Burned, note: None });
        }
        catalog.add_game(g).expect("jogo de demonstração válido");
    }

    let mut extra = HashMap::new();
    let stray = gameini::write(Uuid::new_v4(), "Disco Desconhecido");
    extra.insert("unknown", write_iso(&isos, "DESCONHECIDO.iso", "DESCONHECIDO", &[("GAME.INI", stray.as_bytes())]));
    extra.insert("legacy", write_iso(&isos, "LEGADO.iso", "LEGADO", &[("GAME.INI", b"[Game]\r\nNAME=Half-Life\r\nSTEAMID=70\r\nPROCESS=hl\r\nCOVER=hl.png\r\n")]));
    extra.insert("invalid", write_iso(&isos, "INVALIDO.iso", "INVALIDO", &[("GAME.INI", b"[disc]\r\nid = nao-e-um-uuid\r\n")]));
    extra.insert("noini", write_iso(&isos, "DADOS.iso", "DADOS", &[("LEIAME.TXT", b"um CD de dados qualquer")]));

    (catalog, Demo { dir: dir.to_path_buf(), game_iso, extra })
}

impl Demo {
    pub fn insert_default(&self, _drive: &mut FakeIsoDrive) {}

    pub fn scenario_names(&self) -> Vec<&'static str> {
        vec!["unknown", "legacy", "invalid", "noini", "blank_cdr", "blank_cdrw", "audio", "unreadable", "cdr_used", "cdrw_used"]
    }

    fn iso_of(&self, id: &str) -> Result<PathBuf, String> {
        let gid: Uuid = id.parse().map_err(|_| "game_id inválido")?;
        self.game_iso.get(&gid).cloned().ok_or_else(|| "esse jogo não tem disco de demonstração".to_string())
    }

    pub fn insert(&self, drive: &mut FakeIsoDrive, what: &str) -> Result<(), String> {
        if let Some(id) = what.strip_prefix("game:") {
            drive.insert_iso(self.iso_of(id)?);
            return Ok(());
        }
        if let Some(id) = what.strip_prefix("cdrw_used:") {
            drive.insert_rewritable_iso(self.iso_of(id)?);
            return Ok(());
        }
        match what {
            "blank_cdr" => drive.insert_blank_cdr(),
            "blank_cdrw" => drive.insert_blank_cdrw(),
            "audio" => drive.insert_audio(),
            "unreadable" => drive.insert_unreadable(),
            "cdr_used" => drive.insert_cdr_iso(self.extra["unknown"].clone()),
            "cdrw_used" => drive.insert_rewritable_iso(self.extra["unknown"].clone()),
            other => drive.insert_iso(self.extra.get(other).ok_or_else(|| format!("cenário desconhecido: {other}"))?.clone()),
        }
        Ok(())
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}
