use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VIRTUAL_KEY, VK_DOWN, VK_L, VK_LSHIFT, VK_N, VK_UP,
};

#[inline]
pub fn is_pressed_next() -> bool {
    is_toggled(VK_DOWN) && is_pressed(VK_LSHIFT)
}

#[inline]
pub fn is_pressed_prev() -> bool {
    is_toggled(VK_UP) && is_pressed(VK_LSHIFT)
}

#[inline]
pub fn is_pressed_save() -> bool {
    is_toggled(VK_N) && is_pressed(VK_LSHIFT)
}

#[inline]
pub fn is_pressed_load() -> bool {
    is_toggled(VK_L) && is_pressed(VK_LSHIFT)
}

#[inline]
pub fn is_toggled(c: VIRTUAL_KEY) -> bool {
    (unsafe { GetAsyncKeyState(c.0 as _) & 0x01 }) != 0
}

#[inline]
pub fn is_pressed(c: VIRTUAL_KEY) -> bool {
    (unsafe { (GetAsyncKeyState(c.0 as _) >> 15) & 0x01 }) != 0
}
