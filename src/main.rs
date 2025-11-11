#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use std::cell::Cell;
use std::char::{REPLACEMENT_CHARACTER, decode_utf16};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MessageBoxW};
use windows::core::HSTRING;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CLIP_DEFAULT_PRECIS, COLOR_MENUBAR, CreateFontW, DEFAULT_CHARSET,
            DEFAULT_QUALITY, EndPaint, FF_DONTCARE, GetSysColorBrush, HFONT, OUT_DEFAULT_PRECIS,
            PAINTSTRUCT, SelectObject, SetBkMode, TRANSPARENT, TextOutW,
        },
        UI::{
            Input::KeyboardAndMouse::SetFocus,
            WindowsAndMessaging::{
                CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, ES_CENTER,
                ES_NUMBER, GetMessageW, HMENU, IDI_APPLICATION, IsDialogMessageW, LoadCursorW, MSG,
                PostQuitMessage, RegisterClassW, SW_NORMAL, SendMessageW, ShowWindow,
                TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_CREATE, WM_DESTROY,
                WM_GETTEXT, WM_PAINT, WM_SETFONT, WNDCLASSW, WS_BORDER, WS_CAPTION, WS_CHILD,
                WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    core::{PCWSTR, w},
};

const CLASS_NAME: PCWSTR = w!("iq-calc-window-class");
const ID_EDIT: isize = 42;
const ID_BUTTON: isize = 43;
const TEXT_1: PCWSTR = w!("あなたの IQ を計算します。");
const TEXT_2: PCWSTR = w!("あなたの IQ を入力してください。");

thread_local! {
    static FONT: Cell<HFONT> = Cell::default();
    static EDIT: Cell<HWND> = Cell::default();
    static BUTTON: Cell<HWND> = Cell::default();
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            create(hwnd).ok();
        }
        WM_PAINT => {
            paint(hwnd).ok();
        }
        WM_COMMAND => {
            command(hwnd, wparam).ok();
        }
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
        },
        _ => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    };
    LRESULT::default()
}

fn paint(hwnd: HWND) -> Result<()> {
    let mut ps = PAINTSTRUCT::default();
    unsafe {
        let hdc = BeginPaint(hwnd, &mut ps);
        SelectObject(hdc, FONT.get().into());
        SetBkMode(hdc, TRANSPARENT);
        TextOutW(hdc, 10, 10, TEXT_1.as_wide()).ok()?;
        TextOutW(hdc, 10, 30, TEXT_2.as_wide()).ok()?;
        EndPaint(hwnd, &ps).ok()?;
    };
    Ok(())
}

fn get_edit_string(hwnd: HWND) -> String {
    let mut buf = [0u16; 4];
    unsafe {
        SendMessageW(
            hwnd,
            WM_GETTEXT,
            Some(WPARAM(8)),
            Some(LPARAM(buf.as_mut_ptr() as isize)),
        );
    }
    decode(&buf)
}

fn decode(source: &[u16]) -> String {
    let mut buf = String::with_capacity(source.len() * 2);
    for c in decode_utf16(source.iter().take_while(|&n| n != &0).cloned()) {
        let c = c.unwrap_or(REPLACEMENT_CHARACTER);
        buf.push(c);
    }
    buf
}

fn command(hwnd: HWND, wparam: WPARAM) -> Result<()> {
    let id = loword(wparam.0 as u32) as isize;
    if id == ID_BUTTON {
        let iq = get_edit_string(EDIT.get());
        unsafe {
            MessageBoxW(
                Some(hwnd),
                &HSTRING::from(format!("あなたの IQ は {iq} です！")),
                w!("結果"),
                MB_ICONINFORMATION,
            )
        };
        unsafe { SetFocus(Some(BUTTON.get()))? };
    }
    Ok(())
}

fn create(hwnd: HWND) -> Result<()> {
    create_font();
    create_edit(hwnd)?;
    create_button(hwnd)?;
    Ok(())
}

fn set_font(hwnd: HWND) {
    unsafe { SendMessageW(hwnd, WM_SETFONT, Some(WPARAM(FONT.get().0 as _)), None) };
}

fn create_font() {
    let font = unsafe {
        CreateFontW(
            18,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            FF_DONTCARE.0.into(),
            w!("メイリオ"),
        )
    };
    FONT.set(font);
}

fn create_edit(hwnd: HWND) -> Result<()> {
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("EDIT"),
            None,
            WS_VISIBLE
                | WS_CHILD
                | WS_BORDER
                | WS_TABSTOP
                | WINDOW_STYLE(ES_NUMBER as _)
                | WINDOW_STYLE(ES_CENTER as _),
            210,
            28,
            50,
            22,
            Some(hwnd),
            Some(HMENU(ID_EDIT as _)),
            None,
            None,
        )?
    };
    EDIT.set(hwnd);
    set_font(hwnd);
    unsafe { SetFocus(Some(hwnd))? };
    Ok(())
}

fn create_button(hwnd: HWND) -> Result<()> {
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("計算"),
            WS_VISIBLE | WS_CHILD | WS_TABSTOP,
            80,
            70,
            120,
            25,
            Some(hwnd),
            Some(HMENU(ID_BUTTON as _)),
            None,
            None,
        )?
    };
    set_font(hwnd);
    BUTTON.set(hwnd);
    Ok(())
}

fn main() -> Result<()> {
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        lpszClassName: CLASS_NAME,
        hCursor: unsafe { LoadCursorW(None, IDI_APPLICATION)? },
        hbrBackground: unsafe { GetSysColorBrush(COLOR_MENUBAR) },
        ..Default::default()
    };
    unsafe { RegisterClassW(&wc) };
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CLASS_NAME,
            w!("IQ 計算機"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            300,
            150,
            None,
            None,
            None,
            None,
        )?
    };
    _ = unsafe { ShowWindow(hwnd, SW_NORMAL) };
    let mut msg = MSG::default();
    loop {
        if unsafe { !GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            break;
        }
        if unsafe { !IsDialogMessageW(hwnd, &msg).as_bool() } {
            _ = unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };
        }
    }
    Ok(())
}

#[inline]
fn loword(l: u32) -> u16 {
    (l & 0xffff) as u16
}
