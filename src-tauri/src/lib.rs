use serde_json::{json, Value};
use std::fs;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};

mod css;

#[cfg(target_os = "windows")]
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::Com::*,
    Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*,
};

// Tauri command to update badge count
#[tauri::command]
fn set_zoom(app: AppHandle, zoom: f64) -> std::result::Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let script = format!(
            "document.documentElement.style.zoom = '{}%';",
            (zoom * 100.0).round() as u32
        );
        window.eval(&script).map_err(|e| e.to_string())
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
fn save_zoom(app: AppHandle, zoom: f64) -> std::result::Result<(), String> {
    let app_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;

    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let settings_path = app_dir.join("settings.json");
    let json_data = json!({ "zoomLevel": zoom });

    fs::write(&settings_path, json_data.to_string()).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_zoom(app: AppHandle) -> std::result::Result<Option<f64>, String> {
    let app_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;

    let settings_path = app_dir.join("settings.json");

    if !settings_path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;

    let data: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

    Ok(data.get("zoomLevel").and_then(|v| v.as_f64()))
}

#[tauri::command]
fn set_badge_count(app: AppHandle, count: u32) -> std::result::Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if count > 0 {
            let title = if count > 99 {
                format!("({}) WhatsApp Lite", "99+")
            } else {
                format!("({}) WhatsApp Lite", count)
            };
            let _ = window.set_title(&title);

            // Set taskbar badge on Windows
            #[cfg(target_os = "windows")]
            {
                if let Ok(hwnd) = window.hwnd() {
                    let _ = set_taskbar_badge(hwnd.0 as *mut _, count);
                }
            }
        } else {
            let _ = window.set_title("WhatsApp Lite");

            // Remove taskbar badge on Windows
            #[cfg(target_os = "windows")]
            {
                if let Ok(hwnd) = window.hwnd() {
                    let _ = clear_taskbar_badge(hwnd.0 as *mut _);
                }
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_taskbar_badge(hwnd: *mut std::ffi::c_void, count: u32) -> windows::core::Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        let taskbar: ITaskbarList3 = CoCreateInstance(&TaskbarList, None, CLSCTX_ALL)?;
        taskbar.HrInit()?;

        let hwnd = HWND(hwnd);

        // Create a red circle badge icon
        let badge_icon = create_badge_icon(count)?;

        let desc = w!("New messages");
        ITaskbarList3::SetOverlayIcon(&taskbar, hwnd, badge_icon, desc)?;

        DestroyIcon(badge_icon);
        CoUninitialize();
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn clear_taskbar_badge(hwnd: *mut std::ffi::c_void) -> windows::core::Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        let taskbar: ITaskbarList3 = CoCreateInstance(&TaskbarList, None, CLSCTX_ALL)?;
        taskbar.HrInit()?;

        let hwnd = HWND(hwnd);
        ITaskbarList3::SetOverlayIcon(&taskbar, hwnd, None, w!(""))?;

        CoUninitialize();
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn create_badge_icon(count: u32) -> windows::core::Result<HICON> {
    // 3x5 pixel font for digits 0-9
    #[rustfmt::skip]
    const DIGITS: [[u8; 15]; 10] = [
        [1,1,1, 1,0,1, 1,0,1, 1,0,1, 1,1,1], // 0
        [0,1,0, 1,1,0, 0,1,0, 0,1,0, 1,1,1], // 1
        [1,1,1, 0,0,1, 1,1,1, 1,0,0, 1,1,1], // 2
        [1,1,1, 0,0,1, 1,1,1, 0,0,1, 1,1,1], // 3
        [1,0,1, 1,0,1, 1,1,1, 0,0,1, 0,0,1], // 4
        [1,1,1, 1,0,0, 1,1,1, 0,0,1, 1,1,1], // 5
        [1,1,1, 1,0,0, 1,1,1, 1,0,1, 1,1,1], // 6
        [1,1,1, 0,0,1, 0,1,0, 0,1,0, 0,1,0], // 7
        [1,1,1, 1,0,1, 1,1,1, 1,0,1, 1,1,1], // 8
        [1,1,1, 1,0,1, 1,1,1, 0,0,1, 1,1,1], // 9
    ];

    unsafe {
        let size: i32 = 16;
        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(screen_dc);
        let bitmap = CreateCompatibleBitmap(screen_dc, size, size);
        let old_bitmap = SelectObject(mem_dc, bitmap);

        // Fill background with magenta (will be transparent via mask)
        let magenta = COLORREF(0xFF00FF);
        let magenta_brush = CreateSolidBrush(magenta);
        let bg_rect = RECT {
            left: 0,
            top: 0,
            right: size,
            bottom: size,
        };
        FillRect(mem_dc, &bg_rect, magenta_brush);

        // Draw WhatsApp green filled circle
        let green = COLORREF(0x64D325); // #25D366 in BGR
        let green_brush = CreateSolidBrush(green);
        let green_pen = CreatePen(PS_SOLID, 1, green);
        SelectObject(mem_dc, green_brush);
        SelectObject(mem_dc, green_pen);
        let _ = Ellipse(mem_dc, 0, 0, size, size);

        // Draw white digit pixels using FillRect (pixel art - no font rendering needed)
        let white_brush = CreateSolidBrush(COLORREF(0xFFFFFF));
        let num = count.min(99) as usize;

        if num < 10 {
            // Single digit at 2x scale (6x10 pixels, centered in 16x16)
            let glyph = &DIGITS[num];
            let ox = 5i32; // (16 - 6) / 2 = 5
            let oy = 3i32; // (16 - 10) / 2 = 3
            for row in 0..5i32 {
                for col in 0..3i32 {
                    if glyph[(row * 3 + col) as usize] == 1 {
                        let r = RECT {
                            left: ox + col * 2,
                            top: oy + row * 2,
                            right: ox + col * 2 + 2,
                            bottom: oy + row * 2 + 2,
                        };
                        FillRect(mem_dc, &r, white_brush);
                    }
                }
            }
        } else {
            // Double digit at 1x scale (7x5 pixels, centered in 16x16)
            let d1 = num / 10;
            let d2 = num % 10;
            let ox = 5i32; // (16 - 7) / 2 ≈ 5
            let oy = 6i32; // (16 - 5) / 2 ≈ 6
            for row in 0..5i32 {
                for col in 0..3i32 {
                    if DIGITS[d1][(row * 3 + col) as usize] == 1 {
                        let r = RECT {
                            left: ox + col,
                            top: oy + row,
                            right: ox + col + 1,
                            bottom: oy + row + 1,
                        };
                        FillRect(mem_dc, &r, white_brush);
                    }
                    if DIGITS[d2][(row * 3 + col) as usize] == 1 {
                        let r = RECT {
                            left: ox + 4 + col,
                            top: oy + row,
                            right: ox + 4 + col + 1,
                            bottom: oy + row + 1,
                        };
                        FillRect(mem_dc, &r, white_brush);
                    }
                }
            }
        }

        // Create monochrome mask: magenta pixels → transparent, everything else → opaque
        let mask = CreateBitmap(size, size, 1, 1, None);
        let mask_dc = CreateCompatibleDC(screen_dc);
        let old_mask = SelectObject(mask_dc, mask);
        // When BitBlt from color to mono: pixels matching BkColor → 1 (transparent), others → 0 (opaque)
        SetBkColor(mem_dc, magenta);
        let _ = BitBlt(mask_dc, 0, 0, size, size, mem_dc, 0, 0, SRCCOPY);

        let icon_info = ICONINFO {
            fIcon: TRUE,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: bitmap,
        };
        let icon = CreateIconIndirect(&icon_info)?;

        // Cleanup
        SelectObject(mem_dc, old_bitmap);
        SelectObject(mask_dc, old_mask);
        let _ = DeleteObject(bitmap);
        let _ = DeleteObject(mask);
        let _ = DeleteObject(magenta_brush);
        let _ = DeleteObject(green_brush);
        let _ = DeleteObject(green_pen);
        let _ = DeleteObject(white_brush);
        let _ = DeleteDC(mem_dc);
        let _ = DeleteDC(mask_dc);
        ReleaseDC(None, screen_dc);

        Ok(icon)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_badge_count,
            save_zoom,
            load_zoom,
            set_zoom
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            let handle = app.handle().clone();

            // Load and apply zoom
            if let Ok(app_dir) = handle.path().app_config_dir() {
                let settings_path = app_dir.join("settings.json");
                if settings_path.exists() {
                    if let Ok(contents) = fs::read_to_string(&settings_path) {
                        if let Ok(data) = serde_json::from_str::<Value>(&contents) {
                            if let Some(zoom_f64) = data.get("zoomLevel").and_then(|v| v.as_f64()) {
                                let script = format!(
                                    "document.documentElement.style.zoom = '{}%';",
                                    (zoom_f64 * 100.0).round() as u32
                                );
                                let _ = main_window.eval(&script);
                            }
                        }
                    }
                }
            }

            // System Tray
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show/Hide", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            // Force kill entire process tree including WebView2
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.destroy();
                            }
                            std::process::exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // CSS and JavaScript Injection
            if let Some(window) = app.get_webview_window("main") {
                let main_script = format!(
                    r#"
                    const init = () => {{
                        // Disable Right Click
                        document.addEventListener('contextmenu', event => event.preventDefault());

                        // Inject Custom CSS
                        const css = `{}`;
                        const style = document.createElement('style');
                        style.textContent = css;
                        document.head.append(style);

                        // Monitor for and hide unwanted banners
                        const observer = new MutationObserver((mutations) => {{
                            mutations.forEach((mutation) => {{
                                mutation.addedNodes.forEach((node) => {{
                                    if (node.nodeType === 1) {{
                                        const text = node.innerText;
                                        if (text && (
                                            text.includes('Get WhatsApp for Windows') ||
                                            text.includes('See more chat history') ||
                                            text.includes('Download WhatsApp for Windows') ||
                                            text.includes('Make calls, share your screen')
                                        )) {{
                                            node.style.display = 'none';
                                        }}
                                        const banners = node.querySelectorAll('div, span, a');
                                        banners.forEach(b => {{
                                            const t = b.innerText;
                                            if (t && (
                                                t.includes('Get WhatsApp for Windows') ||
                                                t.includes('See more chat history') ||
                                                t.includes('Download WhatsApp for Windows') ||
                                                t.includes('Make calls, share your screen')
                                            )) {{
                                                const container = b.closest('div[role="button"]') || b.closest('div._aigv') || b.closest('div._aigw') || b;
                                                if (container) container.style.display = 'none';
                                            }}
                                        }});
                                    }}
                                }});
                            }});
                        }});

                        if (document.body) {{
                            observer.observe(document.body, {{ childList: true, subtree: true }});
                        }}

                        // Badge/Notification Count Monitoring
                        let lastCount = 0;
                        const updateBadge = async () => {{
                            try {{
                                let totalUnread = 0;
                                const titleMatch = document.title.match(/\((\d+)\)/);
                                if (titleMatch) {{
                                    totalUnread = parseInt(titleMatch[1]);
                                }} else {{
                                    const badges = document.querySelectorAll('span[data-icon="unread-count"]');
                                    badges.forEach(badge => {{
                                        const count = parseInt(badge.textContent || badge.innerText);
                                        if (!isNaN(count)) totalUnread += count;
                                    }});
                                    if (totalUnread === 0) {{
                                        totalUnread = document.querySelectorAll('div[role="listitem"] span[data-icon="status-unread"]').length;
                                    }}
                                }}

                                if (totalUnread !== lastCount) {{
                                    lastCount = totalUnread;
                                    if (window.__TAURI__) {{
                                        await window.__TAURI__.core.invoke('set_badge_count', {{ count: totalUnread }});
                                    }}
                                }}
                            }} catch (error) {{
                                console.error('Badge update error:', error);
                            }}
                        }};

                        setInterval(updateBadge, 2000);
                        setTimeout(updateBadge, 5000);
                        document.addEventListener('visibilitychange', () => {{
                            if (!document.hidden) setTimeout(updateBadge, 1000);
                        }});
                    }};

                    if (document.readyState === 'loading') {{
                        window.addEventListener('DOMContentLoaded', init);
                    }} else {{
                        init();
                    }}
                    "#,
                    css::CUSTOM_CSS
                );

                // Inject all scripts
                window.eval(&main_script)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
