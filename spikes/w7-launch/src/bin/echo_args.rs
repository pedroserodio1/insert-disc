// Alvo dos testes do W7: grava o diretório de trabalho e cada argumento (entre aspas de depuração).
fn main() {
    let out = std::env::var("W7_OUT").expect("W7_OUT");
    let mut s = format!("cwd={:?}\n", std::env::current_dir().unwrap());
    for a in std::env::args().skip(1) {
        s += &format!("arg={a:?}\n");
    }
    std::fs::write(out, s).unwrap();
}
