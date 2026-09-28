//! `GAME.INI` do disco: leitura defensiva, escrita segura e rótulo do volume.
//! Regras em docs/DISC-FORMAT.md. Do disco só o `id` decide algo (docs/SECURITY.md R1).

use uuid::Uuid;

pub const MAX_INI_BYTES: usize = 4096;
pub const MAX_NAME_CHARS: usize = 64;
/// Limite do IMAPI2 para o nome do volume (docs/DISC-FORMAT.md).
pub const MAX_LABEL_CHARS: usize = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameIni {
    pub id: Uuid,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IniError {
    TooLarge,
    InvalidUtf8,
    /// INI do Reset Floppy Game System (sem `[disc]`, com chaves do original).
    Legacy,
    Invalid,
}

const LEGACY_KEYS: [&str; 5] = ["name", "steamid", "process", "cover", "diskid"];

/// UUID canônico de 36 caracteres (8-4-4-4-12), hexadecimal, não nulo.
fn parse_canonical_uuid(s: &str) -> Option<Uuid> {
    let b = s.as_bytes();
    if b.len() != 36 {
        return None;
    }
    for (i, c) in b.iter().enumerate() {
        let ok = if matches!(i, 8 | 13 | 18 | 23) { *c == b'-' } else { c.is_ascii_hexdigit() };
        if !ok {
            return None;
        }
    }
    Uuid::parse_str(s).ok().filter(|u| !u.is_nil())
}

pub fn parse(bytes: &[u8]) -> Result<GameIni, IniError> {
    if bytes.len() > MAX_INI_BYTES {
        return Err(IniError::TooLarge);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| IniError::InvalidUtf8)?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);

    let mut section = String::new();
    let mut seen_disc = false;
    let mut legacy = false;
    let mut duplicate = false;
    let mut id: Option<&str> = None;
    let mut name: Option<&str> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            if let Some(s) = rest.strip_suffix(']') {
                section = s.trim().to_ascii_lowercase();
                seen_disc |= section == "disc";
            }
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        let k = k.trim().to_ascii_lowercase();
        let v = v.trim();
        if section == "disc" {
            match k.as_str() {
                "id" => duplicate |= id.replace(v).is_some(),
                "name" => duplicate |= name.replace(v).is_some(),
                _ => {}
            }
        } else if LEGACY_KEYS.contains(&k.as_str()) {
            legacy = true;
        }
    }

    if !seen_disc {
        return Err(if legacy { IniError::Legacy } else { IniError::Invalid });
    }
    if duplicate {
        return Err(IniError::Invalid);
    }
    let id = id.and_then(parse_canonical_uuid).ok_or(IniError::Invalid)?;
    let name = name
        .map(|n| n.chars().filter(|c| !c.is_control()).take(MAX_NAME_CHARS).collect::<String>())
        .filter(|n| !n.is_empty());
    Ok(GameIni { id, name })
}

/// Nome como será gravado: sem controles, sem `[` inicial, até 64 caracteres.
pub fn sanitize_name(name: &str) -> String {
    let s: String = name.chars().filter(|c| !c.is_control()).collect();
    s.trim().trim_start_matches('[').trim().chars().take(MAX_NAME_CHARS).collect::<String>().trim().to_string()
}

/// Conteúdo do `GAME.INI` gravado: UTF-8 sem BOM, CRLF, `id` em minúsculas.
pub fn write(id: Uuid, name: &str) -> String {
    format!("[disc]\r\nid = {}\r\nname = {}\r\n", id.hyphenated(), sanitize_name(name))
}

fn fold(c: char) -> Option<char> {
    Some(match c {
        'A'..='Z' | '0'..='9' => c,
        'a'..='z' => c.to_ascii_uppercase(),
        'À'..='Å' | 'à'..='å' => 'A',
        'Ç' | 'ç' => 'C',
        'È'..='Ë' | 'è'..='ë' => 'E',
        'Ì'..='Ï' | 'ì'..='ï' => 'I',
        'Ñ' | 'ñ' => 'N',
        'Ò'..='Ö' | 'ò'..='ö' | 'Ø' | 'ø' => 'O',
        'Ù'..='Ü' | 'ù'..='ü' => 'U',
        'Ý' | 'ý' | 'ÿ' => 'Y',
        _ => return None,
    })
}

/// Rótulo do volume (ISO 9660): `A-Z`, `0-9`, `_`, até 15 caracteres. Regra da Q15.
pub fn sanitize_label(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        match fold(c) {
            Some(f) => out.push(f),
            None if !out.ends_with('_') && !out.is_empty() => out.push('_'),
            None => {}
        }
    }
    let out: String = out.chars().take(MAX_LABEL_CHARS).collect();
    let out = out.trim_end_matches('_').to_string();
    if out.is_empty() { "GAME".into() } else { out }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "6f1c2a4e-9b7d-4c3e-8a51-2d0f7e9b1c44";

    fn ini(s: &str) -> Result<GameIni, IniError> {
        parse(s.as_bytes())
    }

    #[test]
    fn valid_and_roundtrip() {
        let g = ini(&format!("[disc]\nid = {ID}\nname = Hollow Knight\n")).unwrap();
        assert_eq!(g.id.to_string(), ID);
        assert_eq!(g.name.as_deref(), Some("Hollow Knight"));
        let id = Uuid::new_v4();
        let back = parse(write(id, "Celeste").as_bytes()).unwrap();
        assert_eq!((back.id, back.name.as_deref()), (id, Some("Celeste")));
    }

    #[test]
    fn tolerant_syntax() {
        let g = ini(&format!("\u{feff}; c\r\n# c\r\n[ DISC ]\r\n ID = {}\r\n", ID.to_uppercase())).unwrap();
        assert_eq!(g.id.to_string(), ID); // normalizado para minúsculas
        assert_eq!(g.name, None);
    }

    #[test]
    fn extra_keys_and_sections_are_ignored() {
        let g = ini(&format!("[disc]\nid = {ID}\nsteamid = 1\nprocess = evil.exe\n[x]\nid = zzz\n")).unwrap();
        assert_eq!(g.id.to_string(), ID);
    }

    #[test]
    fn rejects_bad_ids() {
        for bad in ["", "{6f1c2a4e-9b7d-4c3e-8a51-2d0f7e9b1c44}", "6f1c2a4e9b7d4c3e8a512d0f7e9b1c44",
                    "00000000-0000-0000-0000-000000000000", "6f1c2a4e-9b7d-4c3e-8a51-2d0f7e9b1c4g"] {
            assert_eq!(ini(&format!("[disc]\nid = {bad}\n")), Err(IniError::Invalid), "{bad}");
        }
        assert_eq!(ini("[disc]\nname = x\n"), Err(IniError::Invalid));
        assert_eq!(ini("id = whatever"), Err(IniError::Invalid));
    }

    #[test]
    fn duplicate_key_is_invalid() {
        assert_eq!(ini(&format!("[disc]\nid = {ID}\nid = {ID}\n")), Err(IniError::Invalid));
    }

    #[test]
    fn legacy_floppy_ini() {
        assert_eq!(ini("[Game]\nNAME=X\nSTEAMID=620\nPROCESS=p\n"), Err(IniError::Legacy));
        assert_eq!(ini("NAME=X\nSTEAMID=620\n"), Err(IniError::Legacy));
    }

    #[test]
    fn size_and_utf8_limits() {
        assert_eq!(parse(&vec![b'a'; MAX_INI_BYTES + 1]), Err(IniError::TooLarge));
        assert_eq!(parse(&[0xff, 0xfe, 0x00]), Err(IniError::InvalidUtf8));
    }

    #[test]
    fn writer_cannot_inject_a_second_id() {
        let evil = "x\r\nid = 11111111-2222-4333-8444-555555555555";
        let id = Uuid::new_v4();
        let parsed = parse(write(id, evil).as_bytes()).unwrap();
        assert_eq!(parsed.id, id);
        assert_eq!(sanitize_name("[[x"), "x");
        assert_eq!(sanitize_name(&"a".repeat(200)).chars().count(), MAX_NAME_CHARS);
    }

    #[test]
    fn labels() {
        assert_eq!(sanitize_label("Hollow Knight"), "HOLLOW_KNIGHT");
        assert_eq!(sanitize_label("Pokémon: Ação!"), "POKEMON_ACAO");
        assert_eq!(sanitize_label("The Legend of Zelda: Breath"), "THE_LEGEND_OF_Z");
        assert_eq!(sanitize_label("???"), "GAME");
        assert_eq!(sanitize_label("  __a__  "), "A");
    }
}
