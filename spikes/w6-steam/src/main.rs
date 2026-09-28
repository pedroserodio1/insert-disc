// Spike W6 (docs/RISKS-AND-SPIKES.md): localizar a Steam, listar bibliotecas e jogos instalados
// (libraryfolders.vdf + appmanifest_*.acf) e achar capas no cache. Só leitura. Descartável.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
enum Vdf {
    Str(String),
    Obj(Vec<(String, Vdf)>),
}

impl Vdf {
    fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Obj(v) => v.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v),
            _ => None,
        }
    }
    fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Vdf::Str(s) => Some(s),
            _ => None,
        }
    }
}

fn tokens(src: &str) -> Vec<String> {
    let (mut out, mut it) = (Vec::new(), src.chars().peekable());
    while let Some(&c) = it.peek() {
        match c {
            c if c.is_whitespace() => { it.next(); }
            '/' => { while it.next_if(|&c| c != '\n').is_some() {} }
            '{' | '}' => { out.push(c.to_string()); it.next(); }
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
            _ => { it.next(); }
        }
    }
    out
}

fn parse_obj(t: &[String], i: &mut usize) -> Vec<(String, Vdf)> {
    let mut v = Vec::new();
    while *i < t.len() && t[*i] != "}" {
        let Some(key) = t[*i].strip_prefix('"') else { *i += 1; continue };
        *i += 1;
        match t.get(*i).map(String::as_str) {
            Some("{") => { *i += 1; let o = parse_obj(t, i); *i += 1; v.push((key.to_string(), Vdf::Obj(o))); }
            Some(s) if s.starts_with('"') => { v.push((key.to_string(), Vdf::Str(s[1..].to_string()))); *i += 1; }
            _ => break,
        }
    }
    v
}

fn parse(src: &str) -> Vdf {
    let t = tokens(src);
    let mut i = 0;
    Vdf::Obj(parse_obj(&t, &mut i))
}

fn steam_path() -> Option<PathBuf> {
    let out = Command::new("reg").args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let line = text.lines().find(|l| l.contains("SteamPath"))?;
    let p = line.split("REG_SZ").nth(1)?.trim();
    Some(PathBuf::from(p))
}

fn steam_uri_handler() -> Option<String> {
    let out = Command::new("reg").args(["query", r"HKCR\steam\shell\open\command", "/ve"]).output().ok()?;
    String::from_utf8_lossy(&out.stdout).lines().find(|l| l.contains("REG_SZ")).map(|l| l.trim().to_string())
}

struct App { id: u32, name: String, state_flags: String }

fn main() {
    let Some(steam) = steam_path() else { println!("Steam: NÃO ENCONTRADA no registro"); return };
    println!("Steam: {}", steam.display());
    println!("Handler steam://: {}", steam_uri_handler().unwrap_or_else(|| "não registrado".into()));

    let lf = steam.join("steamapps").join("libraryfolders.vdf");
    let vdf = parse(&fs::read_to_string(&lf).expect("libraryfolders.vdf"));
    let mut libs: Vec<PathBuf> = Vec::new();
    if let Some(Vdf::Obj(entries)) = vdf.get("libraryfolders") {
        for (_, v) in entries {
            // formato novo: "0" { "path" "..." }; formato antigo: "1" "D:\Lib"
            match v {
                Vdf::Obj(_) => if let Some(p) = v.str("path") { libs.push(PathBuf::from(p)) },
                Vdf::Str(p) if Path::new(p).is_absolute() => libs.push(PathBuf::from(p)),
                _ => {}
            }
        }
    }
    println!("Bibliotecas: {}", libs.len());

    let cache = steam.join("appcache").join("librarycache");
    let mut apps: Vec<App> = Vec::new();
    for lib in &libs {
        let dir = lib.join("steamapps");
        let n_before = apps.len();
        if let Ok(rd) = fs::read_dir(&dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) { continue }
                let Ok(txt) = fs::read_to_string(e.path()) else { continue };
                let acf = parse(&txt);
                let Some(state) = acf.get("AppState") else { continue };
                let (Some(id), Some(nm)) = (state.str("appid").and_then(|s| s.parse().ok()), state.str("name")) else { continue };
                apps.push(App { id, name: nm.to_string(), state_flags: state.str("StateFlags").unwrap_or("?").to_string() });
            }
        }
        println!("  {} -> {} appmanifests", lib.display(), apps.len() - n_before);
    }

    // Steamworks Common Redistributables, Proton etc. também têm appmanifest: filtrar por nome.
    let is_tool = |n: &str| n.starts_with("Steamworks Common") || n.starts_with("Proton") || n.starts_with("Steam Linux Runtime") || n.contains("Redistributable");
    let games: Vec<&App> = apps.iter().filter(|a| !is_tool(&a.name)).collect();
    println!("Jogos instalados (sem ferramentas): {} de {} manifests", games.len(), apps.len());

    let mut layouts: BTreeMap<&str, u32> = BTreeMap::new();
    let mut sample = 0;
    for a in &games {
        let sub = cache.join(a.id.to_string());
        let flat = cache.join(format!("{}_library_600x900.jpg", a.id));
        let kind = if sub.join("library_600x900.jpg").exists() { "subpasta/library_600x900.jpg" }
            else if flat.exists() { "plano/<id>_library_600x900.jpg" }
            else if sub.is_dir() && fs::read_dir(&sub).map(|r| r.flatten().any(|e| image_size(&e.path()).is_some_and(|(w, h)| w >= 300 && h * 2 >= w * 3 - w / 2))).unwrap_or(false) { "outra imagem em retrato (nome hash)" }
            else if sub.is_dir() { "só ícone/logo (sem capa retrato)" }
            else { "sem capa no cache" };
        *layouts.entry(kind).or_default() += 1;
        if sample < 8 {
            println!("  {:>9}  {:<40} flags={} capa={}", a.id, a.name, a.state_flags, kind);
            sample += 1;
        }
    }
    if let Some(a) = games.iter().find(|a| { let s = cache.join(a.id.to_string()); !s.join("library_600x900.jpg").exists() && s.is_dir() }) {
        println!("Exemplo de pasta com nomes hash: {} ({})", a.name, a.id);
        describe_hashed(&cache, a.id);
    }
    println!("Capas no cache local:");
    for (k, v) in &layouts { println!("  {v:>4}  {k}"); }
}

/// Largura e altura de JPEG (marcadores SOF) ou PNG (IHDR), sem dependências.
fn image_size(p: &Path) -> Option<(u32, u32)> {
    let b = fs::read(p).ok()?;
    if b.starts_with(&[0x89, b'P', b'N', b'G']) && b.len() > 24 {
        return Some((u32::from_be_bytes(b[16..20].try_into().ok()?), u32::from_be_bytes(b[20..24].try_into().ok()?)));
    }
    if b.starts_with(&[0xFF, 0xD8]) {
        let mut i = 2;
        while i + 9 < b.len() {
            if b[i] != 0xFF { i += 1; continue }
            let m = b[i + 1];
            if (0xC0..=0xCF).contains(&m) && !matches!(m, 0xC4 | 0xC8 | 0xCC) {
                return Some((u16::from_be_bytes([b[i + 7], b[i + 8]]) as u32, u16::from_be_bytes([b[i + 5], b[i + 6]]) as u32));
            }
            i += 2 + u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        }
    }
    None
}

#[allow(dead_code)]
fn describe_hashed(cache: &Path, id: u32) {
    let mut v: Vec<_> = fs::read_dir(cache.join(id.to_string())).into_iter().flatten().flatten().collect();
    v.sort_by_key(|e| e.file_name());
    for e in v {
        let sz = image_size(&e.path()).map(|(w, h)| format!("{w}x{h}")).unwrap_or("?".into());
        println!("      {} ({} bytes, {sz})", e.file_name().to_string_lossy(), e.metadata().map(|m| m.len()).unwrap_or(0));
    }
}
