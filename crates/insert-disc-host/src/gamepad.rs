//! Fonte de controle no Rust (D1, plano B do W1): lê o controle com `gilrs` e emite os mesmos
//! nomes de navegação da UI (docs/UI-CONTRACT.md, "Entrada de controle"). Só existe com
//! `--features gilrs`; a repetição do direcional e o "segurar" continuam na UI.
//! O app só encaminha estes eventos com a janela em foco (SECURITY R10).

use gilrs::{Axis, Button, EventType, Gilrs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PadEvent {
    /// `up`, `down`, `left`, `right`, `accept`, `back`, `x`, `y`, `lb`, `rb`, `menu`.
    pub name: &'static str,
    pub pressed: bool,
    /// `xbox` ou `playstation` (glifos).
    pub device: &'static str,
}

pub fn button_name(b: Button) -> Option<&'static str> {
    Some(match b {
        Button::South => "accept",
        Button::East => "back",
        Button::West => "x",
        Button::North => "y",
        Button::LeftTrigger => "lb",
        Button::RightTrigger => "rb",
        Button::Start => "menu",
        Button::DPadUp => "up",
        Button::DPadDown => "down",
        Button::DPadLeft => "left",
        Button::DPadRight => "right",
        _ => return None,
    })
}

pub fn device_kind(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if ["dualshock", "dualsense", "playstation", "wireless controller", "054c"].iter().any(|k| n.contains(k)) { "playstation" } else { "xbox" }
}

/// Analógico esquerdo -> direcional, com limiar de 0,5 (igual ao da Gamepad API na UI).
#[derive(Default)]
pub struct StickState {
    x: i8,
    y: i8,
}

impl StickState {
    /// `value` no sentido do gilrs: X positivo = direita, Y positivo = **cima**.
    pub fn update(&mut self, axis: Axis, value: f32, device: &'static str) -> Vec<PadEvent> {
        let dir = if value > 0.5 { 1 } else if value < -0.5 { -1 } else { 0 };
        let (slot, names) = match axis {
            Axis::LeftStickX => (&mut self.x, ["left", "right"]),
            Axis::LeftStickY => (&mut self.y, ["down", "up"]),
            _ => return vec![],
        };
        if *slot == dir {
            return vec![];
        }
        let name = |d: i8| names[usize::from(d > 0)];
        let mut out = Vec::new();
        if *slot != 0 {
            out.push(PadEvent { name: name(*slot), pressed: false, device });
        }
        if dir != 0 {
            out.push(PadEvent { name: name(dir), pressed: true, device });
        }
        *slot = dir;
        out
    }
}

pub struct GamepadSource {
    gilrs: Gilrs,
    stick: StickState,
}

impl GamepadSource {
    /// `None` se o sistema não oferece entrada de controle (o app segue só com a Gamepad API).
    pub fn new() -> Option<Self> {
        Gilrs::new().ok().map(|gilrs| GamepadSource { gilrs, stick: StickState::default() })
    }

    /// Eventos desde a última chamada. Sem controle conectado, volta vazio.
    pub fn poll(&mut self) -> Vec<PadEvent> {
        let mut out = Vec::new();
        while let Some(ev) = self.gilrs.next_event() {
            let device = device_kind(self.gilrs.gamepad(ev.id).name());
            match ev.event {
                EventType::ButtonPressed(b, _) => out.extend(button_name(b).map(|name| PadEvent { name, pressed: true, device })),
                EventType::ButtonReleased(b, _) => out.extend(button_name(b).map(|name| PadEvent { name, pressed: false, device })),
                EventType::AxisChanged(a, v, _) => out.extend(self.stick.update(a, v, device)),
                _ => {}
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_map_to_the_navigation_names() {
        assert_eq!(button_name(Button::South), Some("accept"));
        assert_eq!(button_name(Button::East), Some("back"));
        assert_eq!(button_name(Button::DPadLeft), Some("left"));
        assert_eq!(button_name(Button::Start), Some("menu"));
        assert_eq!(button_name(Button::Mode), None);
    }

    #[test]
    fn stick_uses_a_threshold_and_releases_before_changing_direction() {
        let mut s = StickState::default();
        assert!(s.update(Axis::LeftStickX, 0.3, "xbox").is_empty()); // zona morta
        assert_eq!(s.update(Axis::LeftStickX, 0.9, "xbox"), [PadEvent { name: "right", pressed: true, device: "xbox" }]);
        assert!(s.update(Axis::LeftStickX, 1.0, "xbox").is_empty()); // continua apertado
        let ev = s.update(Axis::LeftStickX, -0.8, "xbox");
        assert_eq!((ev[0].name, ev[0].pressed, ev[1].name, ev[1].pressed), ("right", false, "left", true));
        assert_eq!(s.update(Axis::LeftStickX, 0.0, "xbox")[0], PadEvent { name: "left", pressed: false, device: "xbox" });
        // Y do gilrs: positivo é para cima
        assert_eq!(s.update(Axis::LeftStickY, 0.9, "xbox")[0].name, "up");
        assert_eq!(s.update(Axis::LeftStickY, -0.9, "xbox").last().unwrap().name, "down");
        assert!(s.update(Axis::RightStickX, 1.0, "xbox").is_empty());
    }

    #[test]
    fn device_kind_from_the_controller_name() {
        assert_eq!(device_kind("Sony DualSense Wireless Controller"), "playstation");
        assert_eq!(device_kind("Xbox 360 Controller"), "xbox");
        assert_eq!(device_kind(""), "xbox");
    }

    #[test]
    fn source_starts_and_polls_without_a_controller() {
        if let Some(mut s) = GamepadSource::new() {
            let _ = s.poll(); // sem controle: vazio, sem pânico
        }
    }
}
