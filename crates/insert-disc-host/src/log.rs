//! Log em arquivo (Q23, ADR-0022): rotativo, na pasta de dados. Registra estados, classes e erros;
//! **nunca** a chave de API; caminhos de executável só em nível de depuração
//! (`INSERT_DISC_LOG=debug`).

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 512 * 1024;

pub struct Logger {
    path: PathBuf,
    file: Option<File>,
    pub debug: bool,
}

/// `AAAA-MM-DD HH:MM:SS` em UTC, sem dependências (algoritmo de data civil de Howard Hinnant).
pub fn utc(secs: u64) -> String {
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

impl Logger {
    pub fn open(path: impl Into<PathBuf>, debug: bool) -> Self {
        let path = path.into();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let file = OpenOptions::new().create(true).append(true).open(&path).ok();
        Logger { path, file, debug }
    }

    pub fn info(&mut self, msg: &str) {
        self.write("INFO", msg);
    }

    /// Só com `INSERT_DISC_LOG=debug`: detalhes que podem conter caminhos.
    pub fn debug(&mut self, msg: &str) {
        if self.debug {
            self.write("DEBUG", msg);
        }
    }

    fn write(&mut self, level: &str, msg: &str) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        if self.file.as_ref().and_then(|f| f.metadata().ok()).is_some_and(|m| m.len() >= MAX_BYTES) {
            self.file = None;
            let mut old = self.path.clone().into_os_string();
            old.push(".1");
            let _ = fs::rename(&self.path, PathBuf::from(old)); // mantém só uma geração anterior
            self.file = OpenOptions::new().create(true).append(true).open(&self.path).ok();
        }
        if let Some(f) = &mut self.file {
            let _ = writeln!(f, "{} {level} {}", utc(now), msg.replace('\n', " "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_formats_known_instants() {
        assert_eq!(utc(0), "1970-01-01 00:00:00");
        assert_eq!(utc(951_782_400), "2000-02-29 00:00:00"); // ano bissexto
        assert_eq!(utc(1_759_154_400), "2025-09-29 14:00:00");
    }

    #[test]
    fn debug_lines_only_with_debug_and_the_file_rotates_once() {
        let dir = std::env::temp_dir().join(format!("insert-disc-log-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("logs").join("x.log");
        let mut l = Logger::open(&path, false);
        l.info("estado: LIBRARY");
        l.debug("C:\\Emulators\\emu.exe");
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("INFO estado: LIBRARY") && !text.contains("emu.exe"));

        let mut l = Logger::open(&path, true);
        l.debug("caminho");
        assert!(fs::read_to_string(&path).unwrap().contains("DEBUG caminho"));

        let line = "x".repeat(1000);
        for _ in 0..(MAX_BYTES / 1000 + 5) {
            l.info(&line);
        }
        assert!(path.with_extension("log.1").exists(), "rotacionou");
        assert!(fs::metadata(&path).unwrap().len() < MAX_BYTES);
        let _ = fs::remove_dir_all(&dir);
    }
}
