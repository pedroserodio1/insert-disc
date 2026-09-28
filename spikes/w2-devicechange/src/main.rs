// Spike W2 (docs/RISKS-AND-SPIKES.md): registra WM_DEVICECHANGE de volumes/mídia numa janela
// oculta de nível superior (mensagens de volume são broadcast). Uso: spike-w2-devicechange [segundos]
use std::io::Write;
use std::sync::OnceLock;
use std::time::Instant;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

static START: OnceLock<Instant> = OnceLock::new();

#[repr(C)]
struct Hdr {
    size: u32,
    devicetype: u32,
    reserved: u32,
}

#[repr(C)]
struct Volume {
    size: u32,
    devicetype: u32,
    reserved: u32,
    unitmask: u32,
    flags: u16,
}

fn letters(mask: u32) -> String {
    (0..26).filter(|i| mask & (1 << i) != 0).map(|i| (b'A' + i as u8) as char).map(|c| format!("{c}:")).collect::<Vec<_>>().join(",")
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_DEVICECHANGE => {
            let name = match wp as u32 {
                0x0007 => "DEVNODES_CHANGED",
                0x8000 => "DEVICEARRIVAL",
                0x8001 => "QUERYREMOVE",
                0x8002 => "QUERYREMOVEFAILED",
                0x8003 => "REMOVEPENDING",
                0x8004 => "REMOVECOMPLETE",
                _ => "outro",
            };
            let mut detail = String::new();
            if lp != 0 && matches!(wp as u32, 0x8000..=0x8004) {
                let h = &*(lp as *const Hdr);
                if h.devicetype == 2 {
                    let v = &*(lp as *const Volume);
                    detail = format!("volume {} flags={:#x}{}", letters(v.unitmask), v.flags, if v.flags & 1 != 0 { " (DBTF_MEDIA)" } else { " (dispositivo, não mídia)" });
                } else {
                    detail = format!("devicetype={}", h.devicetype);
                }
            }
            println!("{:>8.3}s {name} {detail}", START.get().unwrap().elapsed().as_secs_f64());
            let _ = std::io::stdout().flush();
            1
        }
        WM_TIMER => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn main() {
    let secs: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(20);
    START.set(Instant::now()).unwrap();
    unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        let class: Vec<u16> = "w2spike\0".encode_utf16().collect();
        let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst, lpszClassName: class.as_ptr(), ..std::mem::zeroed() };
        assert!(RegisterClassW(&wc) != 0);
        let hwnd = CreateWindowExW(0, class.as_ptr(), class.as_ptr(), WS_OVERLAPPED, 0, 0, 10, 10, std::ptr::null_mut(), std::ptr::null_mut(), hinst, std::ptr::null());
        assert!(!hwnd.is_null());
        SetTimer(hwnd, 1, secs * 1000, None);
        println!("{:>8.3}s pronto; escutando por {secs}s", START.get().unwrap().elapsed().as_secs_f64());
        let _ = std::io::stdout().flush();
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
