// Spike W1 (docs/RISKS-AND-SPIKES.md): compara a Gamepad API do WebView2 com o gilrs.
// Descartável: nada aqui é código de produção.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::time::Duration;

use gilrs::{Axis, Event, EventType, GamepadId, Gilrs};
use serde::Serialize;
use tauri::{Emitter, Manager};

#[derive(Clone, Serialize)]
struct PadEvent {
    kind: &'static str,
    detail: String,
    gamepad: String,
    window_focused: bool,
}

// Analógico vira "direcional" ao passar de 0,5; só emite na mudança de zona.
fn axis_zone(v: f32) -> i8 {
    if v > 0.5 {
        1
    } else if v < -0.5 {
        -1
    } else {
        0
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let focused = |h: &tauri::AppHandle| {
                    h.get_webview_window("main")
                        .and_then(|w| w.is_focused().ok())
                        .unwrap_or(false)
                };
                let emit = |kind, detail: String, gamepad: String| {
                    let _ = handle.emit(
                        "gilrs",
                        PadEvent { kind, detail, gamepad, window_focused: focused(&handle) },
                    );
                };

                let mut gilrs = match Gilrs::new() {
                    Ok(g) => g,
                    Err(e) => {
                        emit("error", format!("Gilrs::new falhou: {e}"), String::new());
                        return;
                    }
                };
                for (_, gp) in gilrs.gamepads() {
                    emit("present", format!("{:?}", gp.power_info()), gp.name().to_string());
                }

                let mut zones: HashMap<(GamepadId, Axis), i8> = HashMap::new();
                loop {
                    while let Some(Event { id, event, .. }) = gilrs.next_event() {
                        let name = gilrs.gamepad(id).name().to_string();
                        match event {
                            EventType::ButtonPressed(b, _) => emit("down", format!("{b:?}"), name),
                            EventType::ButtonReleased(b, _) => emit("up", format!("{b:?}"), name),
                            EventType::AxisChanged(a, v, _) => {
                                let z = axis_zone(v);
                                if zones.insert((id, a), z) != Some(z) {
                                    emit("axis", format!("{a:?} {z:+}"), name);
                                }
                            }
                            EventType::Connected => emit("connected", String::new(), name),
                            EventType::Disconnected => emit("disconnected", String::new(), name),
                            _ => {}
                        }
                    }
                    std::thread::sleep(Duration::from_millis(4));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o app");
}
