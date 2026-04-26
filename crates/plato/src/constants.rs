//! Application constants for Plato.

/// Application name.
pub const APP_NAME: &str = "Plato";

/// Framebuffer device path.
pub const FB_DEVICE: &str = "/dev/fb0";

/// RTC device path.
pub const RTC_DEVICE: &str = "/dev/rtc0";

/// Touch input device paths.
pub const TOUCH_INPUTS: [&str; 5] = [
    "/dev/input/by-path/platform-2-0010-event",
    "/dev/input/by-path/platform-1-0038-event",
    "/dev/input/by-path/platform-1-0010-event",
    "/dev/input/by-path/platform-0-0010-event",
    "/dev/input/event1",
];

/// Button input device paths.
pub const BUTTON_INPUTS: [&str; 4] = [
    "/dev/input/by-path/platform-gpio-keys-event",
    "/dev/input/by-path/platform-ntx_event0-event",
    "/dev/input/by-path/platform-mxckpd-event",
    "/dev/input/event0",
];

/// Power input device paths.
pub const POWER_INPUTS: [&str; 3] = [
    "/dev/input/by-path/platform-bd71828-pwrkey.6.auto-event",
    "/dev/input/by-path/platform-bd71828-pwrkey.4.auto-event",
    "/dev/input/by-path/platform-bd71828-pwrkey-event",
];

/// Kobo update bundle path.
pub const KOBO_UPDATE_BUNDLE: &str = "/mnt/onboard/.kobo/KoboRoot.tgz";

/// Clock refresh interval.
pub const CLOCK_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_mins(1);

/// Battery refresh interval.
pub const BATTERY_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(299);

/// Auto suspend refresh interval.
pub const AUTO_SUSPEND_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_mins(1);

/// Suspend wait delay.
pub const SUSPEND_WAIT_DELAY: std::time::Duration = std::time::Duration::from_secs(15);

/// Prepare suspend wait delay.
pub const PREPARE_SUSPEND_WAIT_DELAY: std::time::Duration = std::time::Duration::from_secs(3);
