extern crate winapi;
extern crate user32;

use std::ptr::null_mut;
use winapi::um::winuser::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::processthreadsapi::ExitProcess;
use winapi::shared::minwindef::*;
use winapi::shared::windef::{HWND, HHOOK};

static mut CAPS_HOOK: HHOOK = null_mut();
const EXIT_HOTKEY: i32 = 1; // Ctrl + Alt + L

unsafe fn get_caret_window() -> HWND {
    GetForegroundWindow()
}

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code == HC_ACTION {
        let kbd_struct = &*(l_param as *const KBDLLHOOKSTRUCT);

        if kbd_struct.vkCode == VK_CAPITAL as u32 && GetKeyState(VK_SHIFT) >= 0 {
            if w_param == WM_KEYDOWN as WPARAM {
                let hwnd = get_caret_window();
                if !hwnd.is_null() {
                    PostMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST, 0, 0); 
                    return 1; 
                }
            }
        }
    }

    CallNextHookEx(CAPS_HOOK, n_code, w_param, l_param)
}

unsafe fn show_error(msg: &str) {
    let msg_wide: Vec<u16> = msg.encode_utf16().chain(Some(0)).collect();
    MessageBoxW(null_mut(), msg_wide.as_ptr(), msg_wide.as_ptr(), MB_OK | MB_ICONERROR);
    ExitProcess(1);
}

fn main() {
    unsafe {
        let h_instance = GetModuleHandleW(null_mut());

        if RegisterHotKey(
            null_mut(),
            EXIT_HOTKEY,
            (MOD_CONTROL | MOD_ALT) as u32,
            'L' as u32,
        ) == 0
        {
            show_error("Failed to register hotkey");
        }

        CAPS_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), h_instance, 0);
        if CAPS_HOOK.is_null() {
            show_error("Failed to set hook");
        }

        let mut msg: MSG = std::mem::zeroed();

        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY && msg.wParam == EXIT_HOTKEY as WPARAM {
                PostQuitMessage(0);
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(CAPS_HOOK);
    }
}
