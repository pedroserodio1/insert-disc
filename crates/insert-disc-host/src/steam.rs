//! Steam instalada: localizar, listar jogos e achar capas no cache local (spike W6).
//! Só leitura. O parser de VDF/ACF é puro e testado com fixtures; só `find_steam` toca o sistema.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Vdf {
    Str(String),
    Obj(Vec<(String, Vdf)>),
}

impl Vdf {
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Obj(v) => v.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v),
            Vdf::Str(_) => None,
        }
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Vdf::Str(s) => Some(s),
            Vdf::Obj(_) => None,
        }
    }
}

fn tokens(src: &str) -> Vec<String> {
    let (mut out, mut it) = (Vec::new(), src.chars().peekable());
    while let Some(&c) = it.peek() {
        match c {
            c if c.is_whitespace() => {
                it.next();
            }
            '/' => while it.next_if(|&c| c != '\n').is_some() {},
            '{' | '}' => {
                out.push(c.to_string());
                it.next();
            }
            '"' => {
                it.next();
                let mut s = String::new();
                while let Some(c) = it.next() {
                    match c {
                        '"' => break,
                        '\\' => match it.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(o) => s.push(o),
                            None => {}
                        },
                        c => s.push(c),
                    }
                }
                out.push(format!("\"{s}"));
            }
            _ => {
                it.next();
            }
        }
    }
    out
}

fn parse_obj(t: &[String], i: &mut usize) -> Vec<(String, Vdf)> {
    let mut v = Vec::new();
    while *i < t.len() && t[*i] != "}" {
        let Some(key) = t[*i].strip_prefix('"') else {
            *i += 1;
            continue;
        };
        *i += 1;
        match t.get(*i).map(String::as_str) {
            Some("{") => {
                *i += 1;
                let o = parse_obj(t, i);
                *i += 1;
                v.push((key.to_string(), Vdf::Obj(o)));
            }
            Some(s) if s.starts_with('"') => {
                v.push((key.to_string(), Vdf::Str(s[1..].to_string())));
                *i += 1;
            }
            _ => break,
        }
    }
    v
}

pub fn parse_vdf(src: &str) -> Vdf {
    let t = tokens(src);
    let mut i = 0;
    Vdf::Obj(parse_obj(&t, &mut i))
}

/// Pastas de biblioteca do `libraryfolders.vdf` (formato novo `"0" { "path" ".." }` e antigo `"1" "D:\Lib"`).
pub fn library_folders(vdf_text: &str) -> Vec<PathBuf> {
    let vdf = parse_vdf(vdf_text);
    let mut libs = Vec::new();
    if let Some(Vdf::Obj(entries)) = vdf.get("libraryfolders") {
        for (k, v) in entries {
            if !k.chars().all(|c| c.is_ascii_digit()) {
                continue; // chaves como "contentstatsid"
            }
            match v {
                Vdf::Obj(_) => libs.extend(v.str("path").map(PathBuf::from)),
                Vdf::Str(p) if Path::new(p).is_absolute() => libs.push(PathBuf::from(p)),
                Vdf::Str(_) => {}
            }
        }
    }
    libs
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamGame {
    pub app_id: u32,
    pub name: String,
}

/// `appmanifest_*.acf` -> jogo (None se malformado ou se for ferramenta).
pub fn parse_manifest(acf_text: &str) -> Option<SteamGame> {
    let acf = parse_vdf(acf_text);
    let s = acf.get("AppState")?;
    let app_id: u32 = s.str("appid")?.parse().ok().filter(|id| *id != 0)?;
    let name = s.str("name")?.trim().to_string();
    (!name.is_empty() && !is_tool(&name)).then_some(SteamGame { app_id, name })
}

/// Redistribuíveis, Proton e runtimes também têm appmanifest.
pub fn is_tool(name: &str) -> bool {
    name.starts_with("Steamworks Common") || name.starts_with("Proton") || name.starts_with("Steam Linux Runtime") || name.contains("Redistributable")
}

/// Jogos instalados em todas as bibliotecas de `steam_root`, em ordem alfabética.
pub fn installed_games(steam_root: &Path) -> Vec<SteamGame> {
    let lf = fs::read_to_string(steam_root.join("steamapps").join("libraryfolders.vdf")).unwrap_or_default();
    let mut libs = library_folders(&lf);
    if libs.is_empty() {
        libs.push(steam_root.to_path_buf());
    }
    let mut games: Vec<SteamGame> = Vec::new();
    for lib in libs {
        let Ok(rd) = fs::read_dir(lib.join("steamapps")) else { continue };
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if !(n.starts_with("appmanifest_") && n.ends_with(".acf")) {
                continue;
            }
            if let Some(g) = fs::read_to_string(e.path()).ok().as_deref().and_then(parse_manifest) {
                if !games.iter().any(|x| x.app_id == g.app_id) {
                    games.push(g);
                }
            }
        }
    }
    games.sort_by_key(|g| g.name.to_lowercase());
    games
}

/// Capa retrato no cache da Steam (54% dos jogos têm; o resto fica com placeholder).
pub fn cover_path(steam_root: &Path, app_id: u32) -> Option<PathBuf> {
    let cache = steam_root.join("appcache").join("librarycache");
    [cache.join(app_id.to_string()).join("library_600x900.jpg"), cache.join(format!("{app_id}_library_600x900.jpg"))]
        .into_iter()
        .find(|p| p.is_file())
}

/// Pasta de instalação da Steam (registro do usuário). `None` fora do Windows ou sem Steam.
pub fn find_steam() -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    let out = std::process::Command::new("reg").args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let p = text.lines().find(|l| l.contains("SteamPath"))?.split("REG_SZ").nth(1)?.trim().to_string();
    let p = PathBuf::from(p);
    p.is_dir().then_some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEW: &str = r#""libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"apps" { "228980" "123" }
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
	}
	"contentstatsid"		"-123"
}"#;
    const OLD: &str = "\"LibraryFolders\"\n{\n\t\"TimeNextStatsReport\"\t\"1\"\n\t\"1\"\t\t\"D:\\\\Lib\"\n\t\"2\"\t\t\"relativo\"\n}";

    #[test]
    fn library_folders_new_and_old_formats() {
        assert_eq!(library_folders(NEW), [PathBuf::from(r"C:\Program Files (x86)\Steam"), PathBuf::from(r"D:\SteamLibrary")]);
        assert_eq!(library_folders(OLD), if cfg!(windows) { vec![PathBuf::from(r"D:\Lib")] } else { vec![] });
    }

    #[test]
    fn manifest_gives_a_game_and_skips_tools_and_garbage() {
        let acf = |id: &str, name: &str| format!("\"AppState\"\n{{\n\t\"appid\"\t\t\"{id}\"\n\t\"name\"\t\t\"{name}\"\n\t\"StateFlags\"\t\"4\"\n}}");
        assert_eq!(parse_manifest(&acf("367520", "Hollow Knight")), Some(SteamGame { app_id: 367520, name: "Hollow Knight".into() }));
        assert_eq!(parse_manifest(&acf("228980", "Steamworks Common Redistributables")), None);
        assert_eq!(parse_manifest(&acf("0", "Zero")), None);
        assert_eq!(parse_manifest(&acf("x", "Sem id")), None);
        assert_eq!(parse_manifest("lixo { } \""), None);
    }

    #[test]
    fn installed_games_reads_manifests_from_every_library_and_finds_covers() {
        let root = std::env::temp_dir().join(format!("insert-disc-steam-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let lib2 = root.join("lib2");
        fs::create_dir_all(root.join("steamapps")).unwrap();
        fs::create_dir_all(lib2.join("steamapps")).unwrap();
        let esc = |p: &Path| p.to_string_lossy().replace('\\', "\\\\");
        let vdf = format!("\"libraryfolders\"{{\"0\"{{\"path\"\"{}\"}}\"1\"{{\"path\"\"{}\"}}}}", esc(&root), esc(&lib2));
        fs::write(root.join("steamapps/libraryfolders.vdf"), vdf).unwrap();
        fs::write(root.join("steamapps/appmanifest_20.acf"), "\"AppState\"{\"appid\"\"20\"\"name\"\"Zeta\"}").unwrap();
        fs::write(lib2.join("steamapps/appmanifest_10.acf"), "\"AppState\"{\"appid\"\"10\"\"name\"\"Alfa\"}").unwrap();
        fs::write(root.join("steamapps/appmanifest_99.acf"), "\"AppState\"{\"appid\"\"99\"\"name\"\"Proton 9.0\"}").unwrap();
        let games = installed_games(&root);
        assert_eq!(games.iter().map(|g| g.name.as_str()).collect::<Vec<_>>(), ["Alfa", "Zeta"]);

        assert_eq!(cover_path(&root, 10), None);
        let dir = root.join("appcache/librarycache/10");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("library_600x900.jpg"), b"x").unwrap();
        assert_eq!(cover_path(&root, 10), Some(dir.join("library_600x900.jpg")));
        let _ = fs::remove_dir_all(&root);
    }
}
