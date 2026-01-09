use eframe::egui;

#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, GetWindowTextLengthW, IsWindowVisible, SetForegroundWindow};

#[cfg(target_os = "windows")]
#[derive(Clone)]
struct WindowInfo {
    hwnd: HWND,
    title: String,
}

#[cfg(not(target_os = "windows"))]
#[derive(Clone)]
struct WindowInfo {
    title: String,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd).as_bool() {
        let vec_ptr = lparam.0 as *mut Vec<HWND>;
        if !vec_ptr.is_null() {
            (*vec_ptr).push(hwnd);
        }
    }
    BOOL(1)
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
fn enumerate_windows() -> Vec<WindowInfo> {
    let mut windows_list: Vec<HWND> = Vec::new();
    let ptr = &mut windows_list as *mut _;
    unsafe {
        EnumWindows(Some(enum_proc), LPARAM(ptr as isize));
    }
    
    windows_list.into_iter()
        .map(|hwnd| {
            let title = get_window_title(hwnd);
            WindowInfo { hwnd, title }
        })
        .filter(|win| !win.title.is_empty())
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn enumerate_windows() -> Vec<WindowInfo> {
    // Placeholder for non-Windows platforms
    vec![
        WindowInfo { title: "Demo Window 1".to_string() },
        WindowInfo { title: "Demo Window 2".to_string() },
        WindowInfo { title: "Demo Window 3".to_string() },
    ]
}

struct Switch2App {
    windows: Vec<WindowInfo>,
    filter: String,
    selected_index: Option<usize>,
}

impl Default for Switch2App {
    fn default() -> Self {
        Self {
            windows: enumerate_windows(),
            filter: String::new(),
            selected_index: None,
        }
    }
}

impl eframe::App for Switch2App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Switch2 - Window Switcher");
            
            ui.separator();
            
            // Search/filter box
            ui.horizontal(|ui| {
                ui.label("Search:");
                let response = ui.text_edit_singleline(&mut self.filter);
                
                if response.changed() {
                    self.selected_index = None;
                }
                
                if ui.button("Refresh").clicked() {
                    self.windows = enumerate_windows();
                    self.selected_index = None;
                }
            });
            
            ui.separator();
            
            // Filter windows based on search
            let filter_lower = self.filter.to_lowercase();
            let filtered_windows: Vec<(usize, &WindowInfo)> = self.windows
                .iter()
                .enumerate()
                .filter(|(_, win)| {
                    if self.filter.is_empty() {
                        true
                    } else {
                        win.title.to_lowercase().contains(&filter_lower)
                    }
                })
                .collect();
            
            ui.label(format!("Found {} windows", filtered_windows.len()));
            
            ui.separator();
            
            // Display windows in a scrollable area
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (original_idx, win) in &filtered_windows {
                    let is_selected = self.selected_index == Some(*original_idx);
                    
                    if ui.selectable_label(is_selected, &win.title).clicked() {
                        self.selected_index = Some(*original_idx);
                        // Switch to the window
                        #[cfg(target_os = "windows")]
                        unsafe {
                            SetForegroundWindow(win.hwnd);
                        }
                    }
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Switch2",
        options,
        Box::new(|_cc| Box::<Switch2App>::default()),
    )
}
