//! Teste do `WindowsDrive` com uma ISO montada de verdade (C1). Mexe no Windows (monta e desmonta
//! uma unidade virtual), então fica fora do `cargo test` normal:
//!   cargo test -p insert-disc-host --test windrive -- --ignored
#![cfg(windows)]

use std::process::Command;
use std::time::{Duration, Instant};

use insert_disc_core::drive::*;
use insert_disc_core::{gameini, iso};
use insert_disc_host::windrive::WindowsDrive;
use uuid::Uuid;

fn ps(script: &str) {
    let out = Command::new("powershell").args(["-NoProfile", "-Command", script]).output().expect("powershell");
    assert!(out.status.success(), "{script}: {}", String::from_utf8_lossy(&out.stderr));
}

fn wait<T>(what: &str, mut f: impl FnMut() -> Option<T>) -> T {
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(15) {
        if let Some(v) = f() {
            return v;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("tempo esgotado: {what}");
}

#[test]
#[ignore = "monta uma ISO no Windows"]
fn mounted_iso_appears_reads_like_a_disc_and_disappears() {
    let dir = std::env::temp_dir().join(format!("insert-disc-mount-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let id = Uuid::new_v4();
    let path = dir.join("disco.iso");
    std::fs::write(&path, iso::build_iso("HADES", &[("GAME.INI", gameini::write(id, "Hades").as_bytes())])).unwrap();

    let mut d = WindowsDrive::new();
    let before: Vec<String> = d.list_drives().into_iter().map(|x| x.id).collect();
    d.poll_events(); // estado inicial

    ps(&format!("Mount-DiskImage -ImagePath '{}' | Out-Null", path.display()));
    let (new_id, events) = wait("unidade nova", || {
        let ev = d.poll_events();
        let now = d.list_drives();
        let new = now.iter().find(|x| !before.contains(&x.id))?;
        Some((new.id.clone(), ev))
    });
    let mut events = events;
    if !events.contains(&DriveEvent::DriveAdded) {
        events.extend(wait("DriveAdded", || Some(d.poll_events()).filter(|e| e.contains(&DriveEvent::DriveAdded))));
    }
    assert!(events.contains(&DriveEvent::DriveAdded));

    assert!(d.media_present(&new_id));
    let m = d.read_media(&new_id).expect("lê a mídia");
    assert_eq!((m.kind, m.label.as_deref()), (MediaKind::Data, Some("HADES")));
    assert_eq!(m.ini().expect("GAME.INI válido").id, id);

    // pelo mesmo caminho assíncrono que o núcleo usa
    d.start_op(&new_id, DriveOp::Read, 0);
    let res = wait("leitura assíncrona", || d.poll_ops(0).into_iter().find_map(|u| if let OpUpdate::Done(r) = u { Some(r) } else { None }));
    assert!(matches!(res, OpResult::Read(Ok(m)) if m.ini().is_some_and(|i| i.id == id)));

    ps(&format!("Dismount-DiskImage -ImagePath '{}' | Out-Null", path.display()));
    wait("DriveRemoved", || Some(d.poll_events()).filter(|e| e.contains(&DriveEvent::DriveRemoved)));
    assert!(!d.list_drives().iter().any(|x| x.id == new_id));
    let _ = std::fs::remove_dir_all(&dir);
}
