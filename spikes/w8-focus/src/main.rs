// Spike W8 (docs/RISKS-AND-SPIKES.md): "jogo" fictício com janela própria, para medir como o app
// sai da frente ao lançá-lo e como o usuário volta. Grava em `W8_OUT` (ou stdout) o momento em que
// a janela surgiu e se ela ficou em primeiro plano. Uso: spike-w8-focus [segundos-de-vida]
use std::io::Write;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn main() {
    let secs: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(30);
    let t0 = Instant::now();
    let mut out: Box<dyn Write> = match std::env::var("W8_OUT") {
        Ok(p) => Box::new(std::fs::OpenOptions::new().create(true).append(true).open(p).unwrap()),
        Err(_) => Box::new(std::io::stdout()),
    };
    unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        let class: Vec<u16> = "w8game\0".encode_utf16().collect();
        let title: Vec<u16> = "Jogo Ficticio W8\0".encode_utf16().collect();
        let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst, lpszClassName: class.as_ptr(), hbrBackground: 6 as _, ..std::mem::zeroed() };
        assert!(RegisterClassW(&wc) != 0);
        let hwnd = CreateWindowExW(0, class.as_ptr(), title.as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE, 100, 100, 800, 500, std::ptr::null_mut(), std::ptr::null_mut(), hinst, std::ptr::null());
        assert!(!hwnd.is_null());
        // como um jogo: pede o primeiro plano ao abrir
        let fg_before = GetForegroundWindow();
        SetForegroundWindow(hwnd);
        std::thread::sleep(Duration::from_millis(300));
        let fg = GetForegroundWindow();
        let _ = writeln!(out, "janela criada em {:?}; primeiro plano era a nossa antes de pedir: {}; depois de pedir: {}", t0.elapsed(), fg_before == hwnd, fg == hwnd);
        let _ = out.flush();
        let mut msg: MSG = std::mem::zeroed();
        let end = t0 + Duration::from_secs(secs);
        while Instant::now() < end {
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    return;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
