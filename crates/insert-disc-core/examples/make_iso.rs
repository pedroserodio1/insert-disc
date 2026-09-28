//! Gera uma ISO de teste: `cargo run -p insert-disc-core --example make_iso -- <saida.iso> "<nome do jogo>" [<uuid>]`
use insert_disc_core::{gameini, iso};

fn main() {
    let mut a = std::env::args().skip(1);
    let out = a.next().expect("uso: make_iso <saida.iso> <nome> [uuid]");
    let name = a.next().expect("falta o nome do jogo");
    let id = a.next().map(|s| s.parse().expect("uuid inválido")).unwrap_or_else(uuid::Uuid::new_v4);
    let ini = gameini::write(id, &name);
    std::fs::write(&out, iso::build_iso(&gameini::sanitize_label(&name), &[("GAME.INI", ini.as_bytes())])).unwrap();
    println!("{id}");
}
