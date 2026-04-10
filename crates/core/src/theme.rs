use std::sync::LazyLock;

static DARK_MODE: LazyLock<std::sync::Mutex<bool>> = LazyLock::new(|| std::sync::Mutex::new(false));

#[inline]
pub fn is_dark_mode() -> bool {
    *DARK_MODE.lock().unwrap()
}

#[inline]
pub fn set_dark_mode(enabled: bool) {
    *DARK_MODE.lock().unwrap() = enabled;
}
