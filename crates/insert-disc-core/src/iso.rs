//! ISO 9660 mínimo: leitor (rótulo + um arquivo da raiz) e gerador (uma raiz plana).
//! Serve ao `FakeIsoDrive` (spike W12). Nada de montar imagem no SO.

use std::io::{self, Read, Seek, SeekFrom};

pub const SECTOR: usize = 2048;
const MAX_ROOT_DIR: u32 = 1 << 20;

#[derive(Debug)]
pub enum IsoError {
    Io(io::Error),
    NotIso,
    Corrupt(&'static str),
}

impl From<io::Error> for IsoError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::UnexpectedEof { IsoError::Corrupt("truncada") } else { IsoError::Io(e) }
    }
}

fn read_at<R: Read + Seek>(r: &mut R, off: u64, buf: &mut [u8]) -> Result<(), IsoError> {
    r.seek(SeekFrom::Start(off))?;
    r.read_exact(buf)?;
    Ok(())
}

fn le32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// Lê o rótulo do volume e, se existir na raiz, até `max` bytes do arquivo `name`
/// (sem diferenciar maiúsculas; ignora o sufixo `;1`).
pub fn read_label_and_file<R: Read + Seek>(
    r: &mut R,
    name: &str,
    max: usize,
) -> Result<(String, Option<Vec<u8>>), IsoError> {
    let mut pvd = [0u8; SECTOR];
    read_at(r, 16 * SECTOR as u64, &mut pvd).map_err(|e| match e {
        IsoError::Corrupt(_) => IsoError::NotIso,
        e => e,
    })?;
    if pvd[0] != 1 || &pvd[1..6] != b"CD001" {
        return Err(IsoError::NotIso);
    }
    let label = String::from_utf8_lossy(&pvd[40..72]).trim_end_matches(' ').to_string();

    let root_extent = le32(&pvd, 158);
    let root_size = le32(&pvd, 166);
    if root_size == 0 || root_size > MAX_ROOT_DIR {
        return Err(IsoError::Corrupt("diretório raiz inválido"));
    }
    let mut dir = vec![0u8; root_size as usize];
    read_at(r, root_extent as u64 * SECTOR as u64, &mut dir)?;

    let mut i = 0usize;
    while i < dir.len() {
        let len = dir[i] as usize;
        if len == 0 {
            i = (i / SECTOR + 1) * SECTOR;
            continue;
        }
        if len < 34 || i + len > dir.len() {
            return Err(IsoError::Corrupt("registro de diretório inválido"));
        }
        let flags = dir[i + 25];
        let id_len = dir[i + 32] as usize;
        if 33 + id_len > len {
            return Err(IsoError::Corrupt("nome inválido"));
        }
        let id = &dir[i + 33..i + 33 + id_len];
        let is_special = id_len == 1 && (id[0] == 0 || id[0] == 1);
        if flags & 2 == 0 && !is_special {
            let id = String::from_utf8_lossy(id);
            let id = id.split(';').next().unwrap_or("").trim_end_matches('.');
            if id.eq_ignore_ascii_case(name) {
                let extent = le32(&dir, i + 2) as u64;
                let size = le32(&dir, i + 10) as usize;
                let mut data = vec![0u8; size.min(max)];
                read_at(r, extent * SECTOR as u64, &mut data)?;
                return Ok((label, Some(data)));
            }
        }
        i += len;
    }
    Ok((label, None))
}

fn both32(buf: &mut [u8], at: usize, v: u32) {
    buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
    buf[at + 4..at + 8].copy_from_slice(&v.to_be_bytes());
}

fn both16(buf: &mut [u8], at: usize, v: u16) {
    buf[at..at + 2].copy_from_slice(&v.to_le_bytes());
    buf[at + 2..at + 4].copy_from_slice(&v.to_be_bytes());
}

fn dir_record(extent: u32, size: u32, flags: u8, id: &[u8]) -> Vec<u8> {
    let mut len = 33 + id.len();
    len += len % 2;
    let mut r = vec![0u8; len];
    r[0] = len as u8;
    both32(&mut r, 2, extent);
    both32(&mut r, 10, size);
    r[18..25].copy_from_slice(&[126, 9, 28, 0, 0, 0, 0]);
    r[25] = flags;
    both16(&mut r, 28, 1);
    r[32] = id.len() as u8;
    r[33..33 + id.len()].copy_from_slice(id);
    r
}

fn iso_label(label: &str) -> [u8; 32] {
    let mut out = [b' '; 32];
    for (o, c) in out.iter_mut().zip(label.chars().take(32)) {
        *o = if c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' { c as u8 } else { b'_' };
    }
    out
}

/// Gera uma ISO 9660 com raiz plana. Nomes em 8.3 maiúsculo (ex.: `GAME.INI`).
pub fn build_iso(label: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
    const DATA_START: u32 = 21;
    let mut extents = Vec::new();
    let mut next = DATA_START;
    for (_, data) in files {
        extents.push(next);
        next += data.len().div_ceil(SECTOR).max(1) as u32;
    }
    let total = next as usize;
    let mut img = vec![0u8; total * SECTOR];

    let pvd = &mut img[16 * SECTOR..17 * SECTOR];
    pvd[0] = 1;
    pvd[1..6].copy_from_slice(b"CD001");
    pvd[6] = 1;
    pvd[8..40].fill(b' ');
    pvd[40..72].copy_from_slice(&iso_label(label));
    both32(pvd, 80, total as u32);
    both16(pvd, 120, 1);
    both16(pvd, 124, 1);
    both16(pvd, 128, SECTOR as u16);
    both32(pvd, 132, 10);
    pvd[140..144].copy_from_slice(&18u32.to_le_bytes());
    pvd[148..152].copy_from_slice(&19u32.to_be_bytes());
    pvd[156..190].copy_from_slice(&dir_record(20, SECTOR as u32, 2, &[0]));
    pvd[190..813].fill(b' ');
    for at in [813usize, 830, 864] {
        pvd[at..at + 16].copy_from_slice(b"2026092800000000");
    }
    pvd[847..863].copy_from_slice(b"0000000000000000");
    pvd[881] = 1;

    let term = &mut img[17 * SECTOR..18 * SECTOR];
    term[0] = 255;
    term[1..6].copy_from_slice(b"CD001");
    term[6] = 1;

    let mut lpt = [0u8; 10];
    lpt[0] = 1;
    lpt[2..6].copy_from_slice(&20u32.to_le_bytes());
    lpt[6..8].copy_from_slice(&1u16.to_le_bytes());
    img[18 * SECTOR..18 * SECTOR + 10].copy_from_slice(&lpt);
    let mut mpt = lpt;
    mpt[2..6].copy_from_slice(&20u32.to_be_bytes());
    mpt[6..8].copy_from_slice(&1u16.to_be_bytes());
    img[19 * SECTOR..19 * SECTOR + 10].copy_from_slice(&mpt);

    let mut dir = Vec::new();
    dir.extend(dir_record(20, SECTOR as u32, 2, &[0]));
    dir.extend(dir_record(20, SECTOR as u32, 2, &[1]));
    for ((name, data), extent) in files.iter().zip(&extents) {
        dir.extend(dir_record(*extent, data.len() as u32, 0, format!("{name};1").as_bytes()));
    }
    assert!(dir.len() <= SECTOR, "raiz grande demais para o gerador mínimo");
    img[20 * SECTOR..20 * SECTOR + dir.len()].copy_from_slice(&dir);

    for ((_, data), extent) in files.iter().zip(&extents) {
        let at = *extent as usize * SECTOR;
        img[at..at + data.len()].copy_from_slice(data);
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip() {
        let iso = build_iso("HOLLOW_KNIGHT", &[("GAME.INI", b"[disc]\r\nid = x\r\n"), ("OTHER.TXT", b"zz")]);
        let (label, file) = read_label_and_file(&mut Cursor::new(&iso), "game.ini", 4096).unwrap();
        assert_eq!(label, "HOLLOW_KNIGHT");
        assert_eq!(file.unwrap(), b"[disc]\r\nid = x\r\n");
        let (_, missing) = read_label_and_file(&mut Cursor::new(&iso), "NOPE.INI", 4096).unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn large_file_is_capped() {
        let big = vec![b'a'; 10_000];
        let iso = build_iso("X", &[("GAME.INI", &big)]);
        let (_, f) = read_label_and_file(&mut Cursor::new(&iso), "GAME.INI", 4097).unwrap();
        assert_eq!(f.unwrap().len(), 4097);
    }

    #[test]
    fn corrupt_images_are_errors_not_panics() {
        assert!(matches!(read_label_and_file(&mut Cursor::new(vec![0u8; 100]), "A", 10), Err(IsoError::NotIso)));
        assert!(matches!(read_label_and_file(&mut Cursor::new(vec![0u8; 40_000]), "A", 10), Err(IsoError::NotIso)));
        let iso = build_iso("X", &[("GAME.INI", b"hello")]);
        for cut in [16 * SECTOR + 100, 20 * SECTOR + 10, 21 * SECTOR] {
            let r = read_label_and_file(&mut Cursor::new(&iso[..cut]), "GAME.INI", 100);
            assert!(r.is_err(), "cut {cut}");
        }
        let mut bad = iso.clone();
        bad[20 * SECTOR + 68] = 20; // registro menor que o mínimo (34)
        assert!(read_label_and_file(&mut Cursor::new(&bad), "GAME.INI", 100).is_err());
        // garbage aleatório determinístico
        let mut x = 0x1234_5678u32;
        for _ in 0..200 {
            let mut img = iso.clone();
            for _ in 0..8 {
                x = x.wrapping_mul(1664525).wrapping_add(1013904223);
                let at = (x as usize >> 4) % img.len();
                img[at] = (x >> 24) as u8;
            }
            let _ = read_label_and_file(&mut Cursor::new(&img), "GAME.INI", 100);
        }
    }
}
