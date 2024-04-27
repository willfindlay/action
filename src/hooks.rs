use std::{os::raw::c_int, ptr::null_mut};

use anyhow::Result;
use windows_sys::Win32::{
    Foundation::{GetLastError, HINSTANCE, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageA, SetWindowsHookExA, HHOOK, WH_KEYBOARD_LL, WH_MOUSE_LL,
        WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_SYSKEYDOWN, WM_XBUTTONDOWN,
    },
};

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut()>> = None;

static mut M_HOOK: HHOOK = 0;
static mut K_HOOK: HHOOK = 0;

type RawCallback = unsafe extern "system" fn(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT;

unsafe fn set_mouse_hook(callback: RawCallback) -> Result<()> {
    let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(callback), 0 as HINSTANCE, 0);
    if hook == 0 {
        let error = GetLastError();
        anyhow::bail!("failed to register mouse hook: {}", error);
    }
    M_HOOK = hook;
    Ok(())
}

unsafe fn set_keyboard_hook(callback: RawCallback) -> Result<()> {
    let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(callback), 0 as HINSTANCE, 0);
    if hook == 0 {
        let error = GetLastError();
        anyhow::bail!("failed to register keyboard hook: {}", error);
    }
    K_HOOK = hook;
    Ok(())
}

unsafe extern "system" fn raw_callback(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT {
    if param == WM_LBUTTONDOWN as usize
        || param == WM_RBUTTONDOWN as usize
        || param == WM_XBUTTONDOWN as usize
        || param == WM_MBUTTONDOWN as usize
        || param == WM_KEYDOWN as usize
        || param == WM_SYSKEYDOWN as usize
    {
        #[allow(static_mut_refs)]
        if let Some(callback) = &mut GLOBAL_CALLBACK {
            callback();
        }
    }
    CallNextHookEx(0 as HHOOK, code, param, lpdata)
}

pub fn listen<T>(callback: T) -> Result<()>
where
    T: FnMut() + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        set_keyboard_hook(raw_callback)?;
        set_mouse_hook(raw_callback)?;

        GetMessageA(null_mut(), 0, 0, 0);
    }
    Ok(())
}
