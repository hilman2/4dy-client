#![windows_subsystem = "windows"]

use std::env;
use std::sync::OnceLock;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging as wm;

const WIN_W: i32 = 380;
const WIN_H: i32 = 170;

static NAME: OnceLock<String> = OnceLock::new();
static NUMBER: OnceLock<String> = OnceLock::new();

fn name() -> &'static str {
    NAME.get().map(String::as_str).unwrap_or("")
}

fn number() -> &'static str {
    NUMBER.get().map(String::as_str).unwrap_or("")
}

fn main() {
    fourdy_i18n::init();

    let args: Vec<String> = env::args().collect();
    NAME.set(args.get(1).cloned().unwrap_or("Unbekannt".into()))
        .ok();
    NUMBER.set(args.get(2).cloned().unwrap_or_default()).ok();

    unsafe { run() };
}

unsafe fn run() {
    let inst = GetModuleHandleW(None).unwrap();
    let cls = w!("4dyPopup");

    let wc = wm::WNDCLASSEXW {
        cbSize: size_of::<wm::WNDCLASSEXW>() as u32,
        style: wm::CS_HREDRAW | wm::CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hInstance: inst.into(),
        hCursor: wm::LoadCursorW(None, wm::IDC_ARROW).unwrap(),
        hbrBackground: Gdi::CreateSolidBrush(COLORREF(0x003b291e)),
        lpszClassName: cls,
        ..Default::default()
    };
    wm::RegisterClassExW(&wc);

    let sx = wm::GetSystemMetrics(wm::SM_CXSCREEN);
    let sy = wm::GetSystemMetrics(wm::SM_CYSCREEN);

    let hwnd = wm::CreateWindowExW(
        wm::WS_EX_TOPMOST | wm::WS_EX_TOOLWINDOW,
        cls,
        w!(""),
        wm::WS_POPUP,
        sx - WIN_W - 14,
        sy - WIN_H - 50,
        WIN_W,
        WIN_H,
        Some(HWND::default()),
        Some(wm::HMENU::default()),
        Some(inst.into()),
        None,
    )
    .unwrap();

    let rgn = Gdi::CreateRoundRectRgn(0, 0, WIN_W, WIN_H, 12, 12);
    Gdi::SetWindowRgn(hwnd, Some(rgn), true);

    let _ = wm::ShowWindow(hwnd, wm::SW_SHOWNOACTIVATE);
    let _ = wm::SetForegroundWindow(hwnd);
    wm::SetTimer(Some(hwnd), 9999, 240000, None); // 4 Minuten

    let mut msg = wm::MSG::default();
    while wm::GetMessageW(&mut msg, Some(HWND::default()), 0, 0).into() {
        let _ = wm::TranslateMessage(&msg);
        wm::DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        wm::WM_PAINT => {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let hdc = Gdi::BeginPaint(hwnd, &mut ps);
            paint(hdc);
            let _ = Gdi::EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        wm::WM_LBUTTONDOWN => {
            let y = ((lp.0 >> 16) & 0xFFFF) as i16 as i32;
            let x = (lp.0 & 0xFFFF) as i16 as i32;
            // X-Button oben rechts
            if y <= 26 && x >= WIN_W - 30 {
                let _ = wm::PostMessageW(Some(hwnd), wm::WM_CLOSE, WPARAM(0), LPARAM(0));
            } else if (110..=144).contains(&y) && (16..=WIN_W - 16).contains(&x) {
                do_mailto(hwnd);
            } else {
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                // Fenster verschieben lassen
                let _ = wm::DefWindowProcW(
                    hwnd,
                    wm::WM_NCLBUTTONDOWN,
                    WPARAM(wm::HTCAPTION as usize),
                    lp,
                );
            }
            LRESULT(0)
        }
        wm::WM_TIMER => {
            let _ = wm::PostMessageW(Some(hwnd), wm::WM_CLOSE, WPARAM(0), LPARAM(0));
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            wm::PostQuitMessage(0);
            LRESULT(0)
        }
        _ => wm::DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe fn paint(hdc: Gdi::HDC) {
    let bg = Gdi::CreateSolidBrush(COLORREF(0x003b291e));
    let rc = RECT {
        left: 0,
        top: 0,
        right: WIN_W,
        bottom: WIN_H,
    };
    Gdi::FillRect(hdc, &rc, bg);
    let _ = Gdi::DeleteObject(bg.into());

    // Rote Linie oben
    let accent = Gdi::CreateSolidBrush(COLORREF(0x004444ef));
    let ar = RECT {
        left: 0,
        top: 0,
        right: WIN_W,
        bottom: 3,
    };
    Gdi::FillRect(hdc, &ar, accent);
    let _ = Gdi::DeleteObject(accent.into());

    Gdi::SetBkMode(hdc, Gdi::TRANSPARENT);

    // X-Button oben rechts
    let fx = make_font(-14, 400, "Segoe UI");
    Gdi::SelectObject(hdc, fx.into());
    Gdi::SetTextColor(hdc, COLORREF(0x00646464));
    let mut xrc = RECT {
        left: WIN_W - 30,
        top: 6,
        right: WIN_W - 6,
        bottom: 26,
    };
    let mut xt = to_wide("\u{00D7}");
    Gdi::DrawTextW(hdc, &mut xt, &mut xrc, Gdi::DT_CENTER | Gdi::DT_SINGLELINE);
    let _ = Gdi::DeleteObject(fx.into());

    // Nummer oben (klein, cyan)
    let f1 = make_font(-13, 400, "Consolas");
    Gdi::SelectObject(hdc, f1.into());
    Gdi::SetTextColor(hdc, COLORREF(0x00f8bd38));
    text(hdc, number(), 16, 10);

    // Name mehrzeilig (bis zu 3 Zeilen, weiß, fett)
    let f2 = make_font(-15, 700, "Segoe UI");
    Gdi::SelectObject(hdc, f2.into());
    Gdi::SetTextColor(hdc, COLORREF(0x00ffffff));
    let mut name_w = to_wide(name());
    let mut name_rc = RECT {
        left: 16,
        top: 30,
        right: WIN_W - 16,
        bottom: 100,
    };
    Gdi::DrawTextW(
        hdc,
        &mut name_w,
        &mut name_rc,
        Gdi::DT_LEFT | Gdi::DT_WORDBREAK | Gdi::DT_END_ELLIPSIS,
    );

    // Button
    let btn_top = 110;
    let btn_bottom = 144;
    let btn_brush = Gdi::CreateSolidBrush(COLORREF(0x004444ef));
    let btn_rgn = Gdi::CreateRoundRectRgn(16, btn_top, WIN_W - 16, btn_bottom, 8, 8);
    let _ = Gdi::FillRgn(hdc, btn_rgn, btn_brush);
    let _ = Gdi::DeleteObject(btn_rgn.into());
    let _ = Gdi::DeleteObject(btn_brush.into());

    let f4 = make_font(-13, 600, "Segoe UI");
    Gdi::SelectObject(hdc, f4.into());
    Gdi::SetTextColor(hdc, COLORREF(0x00ffffff));
    let mut btn_rc = RECT {
        left: 16,
        top: btn_top,
        right: WIN_W - 16,
        bottom: btn_bottom,
    };
    let mut t = to_wide(fourdy_i18n::t("popup.send_mail"));
    Gdi::DrawTextW(
        hdc,
        &mut t,
        &mut btn_rc,
        Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE,
    );

    let _ = Gdi::DeleteObject(f1.into());
    let _ = Gdi::DeleteObject(f2.into());
    let _ = Gdi::DeleteObject(f4.into());
}

unsafe fn make_font(size: i32, weight: i32, family: &str) -> Gdi::HFONT {
    let fam = to_wide(family);
    let mut face = [0u16; 32];
    for (i, &ch) in fam.iter().enumerate() {
        if i >= 31 {
            break;
        }
        face[i] = ch;
    }
    Gdi::CreateFontW(
        size,
        0,
        0,
        0,
        weight,
        0,
        0,
        0,
        Gdi::FONT_CHARSET(0),
        Gdi::FONT_OUTPUT_PRECISION(0),
        Gdi::FONT_CLIP_PRECISION(0),
        Gdi::FONT_QUALITY(0),
        0,
        PCWSTR(face.as_ptr()),
    )
}

unsafe fn text(hdc: Gdi::HDC, s: &str, x: i32, y: i32) {
    let mut w = to_wide(s);
    let mut rc = RECT {
        left: x,
        top: y,
        right: x + WIN_W - 32,
        bottom: y + 24,
    };
    Gdi::DrawTextW(
        hdc,
        &mut w,
        &mut rc,
        Gdi::DT_LEFT | Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS,
    );
}

unsafe fn do_mailto(hwnd: HWND) {
    let subject = fourdy_i18n::t("mailto.subject")
        .replace("{name}", name())
        .replace("{number}", number());
    let mailto = format!("mailto:?subject={}", urlencoding::encode(&subject));
    let url = to_wide(&mailto);
    ShellExecuteW(
        Some(hwnd),
        w!("open"),
        PCWSTR(url.as_ptr()),
        None,
        None,
        wm::SW_SHOWNORMAL,
    );
    let _ = wm::PostMessageW(Some(hwnd), wm::WM_CLOSE, WPARAM(0), LPARAM(0));
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
