//! Lançador real (A2, spike W7): executa o `LaunchRequest` do núcleo, sem shell.
//! Argumentos vão como lista (nunca concatenados em uma linha de comando); a URI da Steam só
//! aceita `steam://run/<número>` e é aberta por `explorer.exe` (SECURITY R3).

use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use insert_disc_core::launch::{LaunchError, LaunchRequest};

use crate::steam;

pub fn launch_real(req: &LaunchRequest) -> Result<(), LaunchError> {
    match req {
        LaunchRequest::Uri(u) => {
            let ok = u.strip_prefix("steam://run/").is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()));
            if !ok {
                return Err(LaunchError::Generic("URI não permitida".into()));
            }
            if steam::find_steam().is_none() {
                return Err(LaunchError::SteamMissing);
            }
            let opener = if cfg!(windows) { "explorer.exe" } else { "xdg-open" };
            // ponytail: filho solto (sem wait); no Unix vira zumbi até o app fechar. Coletar se importar.
            Command::new(opener).arg(u).spawn().map(drop).map_err(map_io)
        }
        LaunchRequest::Exec { executable, args, working_dir, elevated } => {
            if !executable.is_file() {
                return Err(LaunchError::NotFound);
            }
            let cwd = working_dir.as_deref().filter(|d| d.is_dir());
            if *elevated {
                return elevated_exec(executable, args, cwd);
            }
            let mut c = Command::new(executable);
            c.args(args);
            if let Some(d) = cwd {
                c.current_dir(d);
            }
            c.spawn().map(drop).map_err(map_io)
        }
    }
}

fn map_io(e: std::io::Error) -> LaunchError {
    match e.kind() {
        ErrorKind::NotFound => LaunchError::NotFound,
        ErrorKind::PermissionDenied => LaunchError::ElevationDenied,
        _ if e.raw_os_error() == Some(740) => LaunchError::ElevationDenied, // ERROR_ELEVATION_REQUIRED
        _ => LaunchError::Generic(e.to_string()),
    }
}

/// Regras de `CommandLineToArgvW`: espaço, tab, aspas e vazio exigem aspas; `\` antes de `"` dobra.
pub fn quote_arg(a: &str) -> String {
    if !a.is_empty() && !a.contains([' ', '\t', '\n', '"']) {
        return a.to_string();
    }
    let mut out = String::from("\"");
    let mut backslashes = 0;
    for c in a.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                out.push_str(&"\\".repeat(backslashes * 2 + 1));
                out.push('"');
                backslashes = 0;
            }
            c => {
                out.push_str(&"\\".repeat(backslashes));
                out.push(c);
                backslashes = 0;
            }
        }
    }
    out.push_str(&"\\".repeat(backslashes * 2));
    out.push('"');
    out
}

#[cfg(windows)]
fn elevated_exec(exe: &Path, args: &[String], cwd: Option<&Path>) -> Result<(), LaunchError> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(hwnd: *mut c_void, op: *const u16, file: *const u16, params: *const u16, dir: *const u16, show: i32) -> isize;
    }
    let wide = |s: &std::ffi::OsStr| s.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
    let params = wide(args.iter().map(|a| quote_arg(a)).collect::<Vec<_>>().join(" ").as_ref());
    let dir = cwd.map(|d| wide(d.as_os_str()));
    // SAFETY: todos os ponteiros vêm de buffers terminados em zero que vivem até o retorno.
    // Não verificado com UAC real (exige interação humana): docs/RISKS-AND-SPIKES.md, W7.
    let r = unsafe {
        ShellExecuteW(std::ptr::null_mut(), wide("runas".as_ref()).as_ptr(), wide(exe.as_os_str()).as_ptr(), params.as_ptr(), dir.as_ref().map_or(std::ptr::null(), |d| d.as_ptr()), 1)
    };
    match r {
        r if r > 32 => Ok(()),
        2 | 3 => Err(LaunchError::NotFound),
        5 => Err(LaunchError::ElevationDenied), // recusou o UAC
        r => Err(LaunchError::Generic(format!("ShellExecute falhou: {r}"))),
    }
}

#[cfg(not(windows))]
fn elevated_exec(_: &Path, _: &[String], _: Option<&Path>) -> Result<(), LaunchError> {
    Err(LaunchError::ElevationDenied) // elevação só existe no Windows
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn quote_arg_follows_the_windows_rules() {
        assert_eq!(quote_arg("simples"), "simples");
        assert_eq!(quote_arg(""), "\"\"");
        assert_eq!(quote_arg("a b"), "\"a b\"");
        assert_eq!(quote_arg("q\"x"), "\"q\\\"x\"");
        assert_eq!(quote_arg("C:\\dir com\\"), "\"C:\\dir com\\\\\"");
        assert_eq!(quote_arg("; calc.exe"), "\"; calc.exe\"");
    }

    #[test]
    fn rejects_uris_other_than_steam_run_number() {
        for u in ["steam://run/", "steam://run/12;calc", "http://x", "steam://rungameid/1", "calc.exe"] {
            assert!(matches!(launch_real(&LaunchRequest::Uri(u.into())), Err(LaunchError::Generic(_))), "{u}");
        }
    }

    #[test]
    fn missing_executable_is_not_found_and_a_real_one_starts() {
        let exec = |exe: PathBuf, args: Vec<String>| LaunchRequest::Exec { executable: exe, args, working_dir: None, elevated: false };
        let gone = std::env::temp_dir().join("insert-disc-nao-existe.exe");
        assert_eq!(launch_real(&exec(gone, vec![])), Err(LaunchError::NotFound));

        let (exe, args) = if cfg!(windows) {
            (PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into())).join(r"System32\cmd.exe"), vec!["/c".to_string(), "exit".into(), "0".into()])
        } else {
            (PathBuf::from("/bin/true"), vec![])
        };
        assert_eq!(launch_real(&exec(exe, args)), Ok(()));
    }
}
