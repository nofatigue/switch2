use std::env;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, GetWindowTextLengthW, IsWindowVisible, SetForegroundWindow};

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd).as_bool() {
        let vec_ptr = lparam.0 as *mut Vec<HWND>;
        if !vec_ptr.is_null() {
            (*vec_ptr).push(hwnd);
        }
    }
    BOOL(1)
}

fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
        let read = GetWindowTextW(hwnd, &mut buf);
        if read <= 0 {
            return String::new();
        }
        OsString::from_wide(&buf[..read as usize]).to_string_lossy().into_owned()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Enumerate windows
    let mut windows_list: Vec<HWND> = Vec::new();
    let ptr = &mut windows_list as *mut _;
    unsafe {
        EnumWindows(Some(enum_proc), LPARAM(ptr as isize));
    }

    if args.len() == 1 {
        // List windows
        for (i, hwnd) in windows_list.iter().enumerate() {
            let title = get_window_title(*hwnd);
            if !title.is_empty() {
                println!("{}: {} (hwnd: {:?})", i, title, hwnd);
            }
        }
        println!("\nRun with an index or substring to switch to that window:  switch2 <index|substring>");
        return;
    }

    let query = args[1..].join(" ");

    // Try parse as index
    if let Ok(idx) = query.parse::<usize>() {
        if idx < windows_list.len() {
            unsafe { SetForegroundWindow(windows_list[idx]); }
            return;
        } else {
            eprintln!("Index out of range");
            return;
        }
    }

    // Otherwise match substring
    let q_lower = query.to_lowercase();
    for hwnd in windows_list {
        let title = get_window_title(hwnd);
        if title.to_lowercase().contains(&q_lower) {
            unsafe { SetForegroundWindow(hwnd); }
            return;
        }
    }

    eprintln!("No matching window found for '{}'", query);
}
