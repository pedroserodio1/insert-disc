// Spike W7 (docs/RISKS-AND-SPIKES.md): lançar executáveis com argumentos, sem shell.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(exe: &Path, args: &[&str], cwd: Option<&Path>, out: &Path) -> Result<String, String> {
    let _ = fs::remove_file(out);
    let mut c = Command::new(exe);
    c.args(args).env("W7_OUT", out);
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    let status = c.status().map_err(|e| format!("erro ao iniciar: kind={:?} {e}", e.kind()))?;
    if !status.success() {
        return Err(format!("saiu com {status}"));
    }
    fs::read_to_string(out).map_err(|e| format!("sem saída: {e}"))
}

fn main() {
    let tmp = std::env::temp_dir().join("w7-launch test");
    let _ = fs::remove_dir_all(&tmp);
    let dir = tmp.join("pasta com espaços");
    fs::create_dir_all(&dir).unwrap();
    let helper = std::env::current_exe().unwrap().with_file_name("echo_args.exe");
    let exe = dir.join("meu jogo.exe");
    fs::copy(&helper, &exe).expect("compile também o echo_args (cargo build)");
    let out = tmp.join("out.txt");

    println!("== 1. Caminho com espaços e argumentos hostis (esperado: cada um chega como UM argumento)");
    let hostile = ["--rom", r"C:\roms\a b.sfc", "quote\"aspas", "; calc.exe", "& whoami", "%PATH%", "^caret", "tab\there", "vazio:", ""];
    match run(&exe, &hostile, None, &out) {
        Ok(got) => {
            let want: Vec<String> = hostile.iter().map(|a| format!("arg={a:?}")).collect();
            let recv: Vec<&str> = got.lines().filter(|l| l.starts_with("arg=")).collect();
            println!("   recebido == enviado: {}", recv == want.iter().map(String::as_str).collect::<Vec<_>>());
            if recv != want.iter().map(String::as_str).collect::<Vec<_>>() { println!("{got}"); }
        }
        Err(e) => println!("   FALHA: {e}"),
    }

    println!("== 2. Diretório de trabalho");
    let cwd_case = tmp.join("outro cwd");
    fs::create_dir_all(&cwd_case).unwrap();
    for (label, cwd) in [("padrão (herda do launcher)", None), ("explícito", Some(cwd_case.as_path())), ("pasta do executável", Some(dir.as_path()))] {
        match run(&exe, &[], cwd, &out) {
            Ok(got) => println!("   {label}: {}", got.lines().next().unwrap_or("")),
            Err(e) => println!("   {label}: FALHA {e}"),
        }
    }

    println!("== 3. Executável inexistente");
    match run(&dir.join("nao existe.exe"), &[], None, &out) {
        Ok(_) => println!("   INESPERADO: executou"),
        Err(e) => println!("   {e}"),
    }

    println!("== 4. .bat com argumentos (Q13): o Rust protege ou recusa?");
    // Caminho ASCII: o cmd.exe lê o .bat na página de código OEM, então "ç" no caminho quebra (achado).
    let bat_dir = tmp.join("bat dir");
    fs::create_dir_all(&bat_dir).unwrap();
    let bat_exe = bat_dir.join("game.exe");
    fs::copy(&helper, &bat_exe).unwrap();
    let bat = bat_dir.join("run.bat");
    fs::write(&bat, format!("@echo off\r\n\"{}\" %*\r\n", bat_exe.display())).unwrap();
    for arg in ["simples", "com espaço", "& echo INJETADO", "\"& echo INJETADO", "%PATH%", "linha1\nlinha2"] {
        match run(&bat, &[arg], None, &out) {
            Ok(got) => println!("   {arg:?} -> {:?}", got.lines().filter(|l| l.starts_with("arg=")).collect::<Vec<_>>()),
            Err(e) => println!("   {arg:?} -> RECUSADO/FALHOU: {e}"),
        }
    }

    println!("== 5. Não bloquear o app: spawn e soltar");
    let t = std::time::Instant::now();
    let mut child = Command::new("cmd.exe").args(["/c", "ping", "-n", "3", "127.0.0.1"]).stdout(std::process::Stdio::null()).spawn().unwrap();
    println!("   spawn retornou em {:?}; filho ainda rodando: {}", t.elapsed(), child.try_wait().unwrap().is_none());
    let _ = child.wait();

    println!("== 6. Steam: URI aberta por explorer.exe (não executada aqui, para não abrir jogo)");
    println!("   Comando previsto: Command::new(\"explorer.exe\").arg(\"steam://run/<AppID>\")  [NÃO VERIFICADO]");
    println!("== 7. Elevação (UAC): exige interação humana; NÃO VERIFICADO");
    let _: PathBuf = tmp;
}
