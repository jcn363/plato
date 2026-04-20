use crate::device::CURRENT_DEVICE;
use crate::framebuffer::Display;
use crate::geom::{LinearDir, Point};
use crate::log_error;
use crate::settings::ButtonScheme;
use anyhow::{Context, Error};
use rustc_hash::FxHashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::mem::{self, MaybeUninit};
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::slice;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

// Event types
pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_ABS: u16 = 0x03;
pub const EV_MSC: u16 = 0x04;

// Event codes
pub const ABS_MT_TRACKING_ID: u16 = 0x39;
pub const ABS_MT_POSITION_X: u16 = 0x35;
pub const ABS_MT_POSITION_Y: u16 = 0x36;
pub const ABS_MT_PRESSURE: u16 = 0x3a;
pub const ABS_MT_TOUCH_MAJOR: u16 = 0x30;
pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;
pub const ABS_PRESSURE: u16 = 0x18;
pub const MSC_RAW: u16 = 0x03;
pub const SYN_REPORT: u16 = 0x00;

// Event values
pub const MSC_RAW_GSENSOR_PORTRAIT_DOWN: i32 = 0x17;
pub const MSC_RAW_GSENSOR_PORTRAIT_UP: i32 = 0x18;
pub const MSC_RAW_GSENSOR_LANDSCAPE_RIGHT: i32 = 0x19;
pub const MSC_RAW_GSENSOR_LANDSCAPE_LEFT: i32 = 0x1a;

// The indices of this clockwise ordering of the sensor values match the Forma's rotation values.
pub const GYROSCOPE_ROTATIONS: [i32; 4] = [
    MSC_RAW_GSENSOR_LANDSCAPE_LEFT,
    MSC_RAW_GSENSOR_PORTRAIT_UP,
    MSC_RAW_GSENSOR_LANDSCAPE_RIGHT,
    MSC_RAW_GSENSOR_PORTRAIT_DOWN,
];

pub const VAL_RELEASE: i32 = 0;
pub const VAL_PRESS: i32 = 1;
pub const VAL_REPEAT: i32 = 2;

// Key codes
pub const KEY_POWER: u16 = 116;
pub const KEY_HOME: u16 = 102;
pub const KEY_LIGHT: u16 = 90;
pub const KEY_BACKWARD: u16 = 193;
pub const KEY_FORWARD: u16 = 194;
pub const PEN_ERASE: u16 = 331;
pub const PEN_HIGHLIGHT: u16 = 332;
pub const SLEEP_COVER: [u16; 2] = [59, 35];
// Synthetic touch button
pub const BTN_TOUCH: u16 = 330;
// The following key codes are fake, and are used to support
// software toggles within this design
pub const KEY_ROTATE_DISPLAY: u16 = 0xffff;
pub const KEY_BUTTON_SCHEME: u16 = 0xfffe;

pub const SINGLE_TOUCH_CODES: TouchCodes = TouchCodes {
    pressure: ABS_PRESSURE,
    x: ABS_X,
    y: ABS_Y,
};

pub const MULTI_TOUCH_CODES_A: TouchCodes = TouchCodes {
    pressure: ABS_MT_TOUCH_MAJOR,
    x: ABS_MT_POSITION_X,
    y: ABS_MT_POSITION_Y,
};

pub const MULTI_TOUCH_CODES_B: TouchCodes = TouchCodes {
    pressure: ABS_MT_PRESSURE,
    ..MULTI_TOUCH_CODES_A
};

#[repr(C)]
pub struct InputEvent {
    pub time: libc::timeval,
    pub kind: u16, // type
    pub code: u16,
    pub value: i32,
}

// Handle different touch protocols
#[derive(Debug)]
pub struct TouchCodes {
    pressure: u16,
    x: u16,
    y: u16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TouchProto {
    Single,
    MultiA,
    MultiB, // Pressure won't indicate a finger release.
    MultiC,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FingerStatus {
    Down,
    Motion,
    Move,
    Up,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ButtonStatus {
    Pressed,
    Released,
    Repeated,
}

impl ButtonStatus {
    #[inline]
    pub fn try_from_raw(value: i32) -> Option<ButtonStatus> {
        match value {
            VAL_RELEASE => Some(ButtonStatus::Released),
            VAL_PRESS => Some(ButtonStatus::Pressed),
            VAL_REPEAT => Some(ButtonStatus::Repeated),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ButtonCode {
    Power,
    Home,
    Light,
    Backward,
    Forward,
    Erase,
    Highlight,
    Raw(u16),
}

impl ButtonCode {
    fn from_raw(code: u16, rotation: i8, button_scheme: ButtonScheme) -> ButtonCode {
        match code {
            KEY_POWER => ButtonCode::Power,
            KEY_HOME => ButtonCode::Home,
            KEY_LIGHT => ButtonCode::Light,
            KEY_BACKWARD => resolve_button_direction(LinearDir::Backward, rotation, button_scheme),
            KEY_FORWARD => resolve_button_direction(LinearDir::Forward, rotation, button_scheme),
            PEN_ERASE => ButtonCode::Erase,
            PEN_HIGHLIGHT => ButtonCode::Highlight,
            _ => ButtonCode::Raw(code),
        }
    }
}

#[inline]
fn resolve_button_direction(
    mut direction: LinearDir,
    rotation: i8,
    button_scheme: ButtonScheme,
) -> ButtonCode {
    if (CURRENT_DEVICE.should_invert_buttons(rotation)) ^ (button_scheme == ButtonScheme::Inverted)
    {
        direction = direction.opposite();
    }

    if direction == LinearDir::Forward {
        return ButtonCode::Forward;
    }

    ButtonCode::Backward
}

pub fn display_rotate_event(n: i8) -> InputEvent {
    let mut tp = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    // SAFETY: libc::gettimeofday writes to a valid timeval struct. The second argument is null, which is allowed.
    unsafe {
        libc::gettimeofday(&mut tp, ptr::null_mut());
    }
    InputEvent {
        time: tp,
        kind: EV_KEY,
        code: KEY_ROTATE_DISPLAY,
        value: n as i32,
    }
}

pub fn button_scheme_event(v: i32) -> InputEvent {
    let mut tp = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    // SAFETY: libc::gettimeofday writes to a valid timeval struct. The second argument is null, which is allowed.
    unsafe {
        libc::gettimeofday(&mut tp, ptr::null_mut());
    }
    InputEvent {
        time: tp,
        kind: EV_KEY,
        code: KEY_BUTTON_SCHEME,
        value: v,
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DeviceEvent {
    Finger {
        id: i32,
        time: f64,
        status: FingerStatus,
        position: Point,
    },
    Button {
        time: f64,
        code: ButtonCode,
        status: ButtonStatus,
    },
    Keyboard {
        time: f64,
        code: u32,
        status: ButtonStatus,
    },
    Plug(PowerSource),
    Unplug(PowerSource),
    RotateScreen(i8),
    CoverOn,
    CoverOff,
    NetUp,
    UserActivity,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PowerSource {
    Host,
    Wall,
}

pub fn seconds(time: libc::timeval) -> f64 {
    time.tv_sec as f64 + time.tv_usec as f64 / 1e6
}

pub fn raw_events(paths: Vec<String>) -> (Sender<InputEvent>, Receiver<InputEvent>) {
    // Input validation: ensure paths vector is not empty
    if paths.is_empty() {
        log_error!("Cannot create raw events: paths vector is empty");
        let (tx, rx) = mpsc::channel();
        return (tx, rx);
    }

    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    thread::spawn(move || parse_raw_events(&paths, &tx));
    (tx2, rx)
}

pub fn parse_raw_events(paths: &[String], tx: &Sender<InputEvent>) -> Result<(), Error> {
    let mut files = Vec::with_capacity(paths.len());
    let mut pfds = Vec::with_capacity(paths.len());

    for path in paths.iter() {
        let file = File::open(path).with_context(|| format!("can't open input file {}", path))?;
        let fd = file.as_raw_fd();
        files.push(file);
        pfds.push(libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        });
    }

    loop {
        // SAFETY: libc::poll called with valid pollfd array pointer and length.
        let ret = unsafe { libc::poll(pfds.as_mut_ptr(), pfds.len() as libc::nfds_t, -1) };
        if ret < 0 {
            break;
        }
        for (pfd, mut file) in pfds.iter().zip(&files) {
            if pfd.revents & libc::POLLIN != 0 {
                let mut input_event = MaybeUninit::<InputEvent>::uninit();
                // SAFETY: MaybeUninit pointer is valid. slice::from_raw_parts_mut creates a byte slice matching InputEvent size. assume_init is safe after reading fills the bytes.
                unsafe {
                    let event_slice = slice::from_raw_parts_mut(
                        input_event.as_mut_ptr() as *mut u8,
                        mem::size_of::<InputEvent>(),
                    );
                    if file.read_exact(event_slice).is_err() {
                        break;
                    }
                    tx.send(input_event.assume_init()).ok();
                }
            }
        }
    }

    Ok(())
}

pub fn usb_events() -> Receiver<DeviceEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || parse_usb_events(&tx));
    rx
}

fn parse_usb_events(tx: &Sender<DeviceEvent>) {
    let path = match CString::new("/tmp/nickel-hardware-status") {
        Ok(p) => p,
        Err(_) => return,
    };
    // SAFETY: CString is valid and null-terminated. libc::open opens a FIFO with valid flags.
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_NONBLOCK | libc::O_RDWR) };

    if fd < 0 {
        return;
    }

    let mut pfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };

    const BUF_LEN: usize = 256;

    loop {
        // SAFETY: libc::poll called with valid pollfd pointer and count of 1.
        let ret = unsafe { libc::poll(&mut pfd as *mut libc::pollfd, 1, -1) };

        if ret < 0 {
            break;
        }

        let buf = match CString::new(vec![1; BUF_LEN]) {
            Ok(b) => b,
            Err(_) => break,
        };
        let c_buf = buf.into_raw();

        if pfd.revents & libc::POLLIN != 0 {
            // SAFETY: libc::read with valid file descriptor and buffer pointer. CString::from_raw reclaims ownership of the pointer previously obtained via into_raw.
            let n = unsafe { libc::read(fd, c_buf as *mut libc::c_void, BUF_LEN as libc::size_t) };
            // SAFETY: c_buf was obtained from CString::into_raw. from_raw reclaims ownership and will deallocate properly.
            let buf = unsafe { CString::from_raw(c_buf) };
            if n > 0 {
                if let Ok(s) = buf.to_str() {
                    handle_usb_messages(s, n as usize, tx);
                }
            } else {
                break;
            }
        }
    }
}

fn handle_usb_messages(s: &str, len: usize, tx: &Sender<DeviceEvent>) {
    for msg in s[..len].lines() {
        if msg == "usb plug add" {
            tx.send(DeviceEvent::Plug(PowerSource::Host)).ok();
        } else if msg == "usb plug remove" {
            tx.send(DeviceEvent::Unplug(PowerSource::Host)).ok();
        } else if msg == "usb ac add" {
            tx.send(DeviceEvent::Plug(PowerSource::Wall)).ok();
        } else if msg == "usb ac remove" {
            tx.send(DeviceEvent::Unplug(PowerSource::Wall)).ok();
        } else if msg.starts_with("network bound") {
            tx.send(DeviceEvent::NetUp).ok();
        }
    }
}

pub fn device_events(
    rx: Receiver<InputEvent>,
    display: Display,
    button_scheme: ButtonScheme,
) -> Receiver<DeviceEvent> {
    // Input validation: ensure display dimensions are valid
    if display.dims.0 == 0 || display.dims.1 == 0 {
        log_error!("Invalid display dimensions: {:?}", display.dims);
        let (_ty, ry) = mpsc::channel();
        return ry;
    }

    let (ty, ry) = mpsc::channel();
    thread::spawn(move || parse_device_events(&rx, &ty, display, button_scheme));
    ry
}

#[derive(Default)]
struct TouchState {
    position: Point,
    pressure: i32,
}

pub fn parse_device_events(
    rx: &Receiver<InputEvent>,
    ty: &Sender<DeviceEvent>,
    display: Display,
    button_scheme: ButtonScheme,
) {
    let mut id = 0;
    let mut last_activity = -60;
    let Display {
        mut dims,
        mut rotation,
    } = display;
    let mut fingers: FxHashMap<i32, Point> = FxHashMap::default();
    let mut packets: FxHashMap<i32, TouchState> = FxHashMap::default();
    let proto = CURRENT_DEVICE.proto;

    let mut tc = initialize_touch_codes(proto);
    initialize_single_touch_packet(proto, id, &mut packets);

    let (mut mirror_x, mut mirror_y) = CURRENT_DEVICE.should_mirror_axes(rotation);
    if CURRENT_DEVICE.should_swap_axes(rotation) {
        mem::swap(&mut tc.x, &mut tc.y);
    }

    let mut button_scheme = button_scheme;

    while let Ok(evt) = rx.recv() {
        if evt.kind == EV_ABS {
            handle_abs_event(
                evt,
                &mut id,
                &mut packets,
                &tc,
                mirror_x,
                mirror_y,
                dims,
                proto,
            );
        } else if evt.kind == EV_SYN && evt.code == SYN_REPORT {
            handle_syn_event(
                evt,
                &mut last_activity,
                &mut fingers,
                &mut packets,
                proto,
                ty,
            );
        } else if evt.kind == EV_KEY {
            handle_key_event(
                evt,
                &mut button_scheme,
                &mut rotation,
                &mut tc,
                &mut dims,
                &mut mirror_x,
                &mut mirror_y,
                ty,
            );
        } else if evt.kind == EV_MSC && evt.code == MSC_RAW {
            handle_msc_event(evt, ty);
        }
    }
}

#[inline]
fn initialize_touch_codes(proto: TouchProto) -> TouchCodes {
    match proto {
        TouchProto::Single => SINGLE_TOUCH_CODES,
        TouchProto::MultiA => MULTI_TOUCH_CODES_A,
        TouchProto::MultiB => MULTI_TOUCH_CODES_B,
        TouchProto::MultiC => MULTI_TOUCH_CODES_B,
    }
}

#[inline]
fn initialize_single_touch_packet(
    proto: TouchProto,
    id: i32,
    packets: &mut FxHashMap<i32, TouchState>,
) {
    if proto == TouchProto::Single {
        packets.insert(id, TouchState::default());
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_abs_event(
    evt: InputEvent,
    id: &mut i32,
    packets: &mut FxHashMap<i32, TouchState>,
    tc: &TouchCodes,
    mirror_x: bool,
    mirror_y: bool,
    dims: (u32, u32),
    proto: TouchProto,
) {
    if evt.code == ABS_MT_TRACKING_ID {
        if evt.value >= 0 {
            *id = evt.value;
            packets.insert(*id, TouchState::default());
        }
    } else if evt.code == tc.x {
        if let Some(state) = packets.get_mut(id) {
            state.position.x = if mirror_x {
                dims.0 as i32 - 1 - evt.value
            } else {
                evt.value
            };
        }
    } else if evt.code == tc.y {
        if let Some(state) = packets.get_mut(id) {
            state.position.y = if mirror_y {
                dims.1 as i32 - 1 - evt.value
            } else {
                evt.value
            };
        }
    } else if evt.code == tc.pressure {
        if let Some(state) = packets.get_mut(id) {
            state.pressure = evt.value;
            if proto == TouchProto::Single && CURRENT_DEVICE.mark() == 3 && state.pressure == 0 {
                state.position.x = dims.0 as i32 - 1 - state.position.x;
                mem::swap(&mut state.position.x, &mut state.position.y);
            }
        }
    }
}

fn handle_syn_event(
    evt: InputEvent,
    last_activity: &mut i64,
    fingers: &mut FxHashMap<i32, Point>,
    packets: &mut FxHashMap<i32, TouchState>,
    proto: TouchProto,
    ty: &Sender<DeviceEvent>,
) {
    let tv_sec = evt.time.tv_sec;
    if (tv_sec - *last_activity).abs() >= 60 {
        *last_activity = tv_sec;
        ty.send(DeviceEvent::UserActivity).ok();
    }

    if proto == TouchProto::MultiB {
        retain_multi_b_fingers(fingers, packets, evt.time, ty);
    }

    handle_finger_state_changes(fingers, packets, evt.time, ty);

    if proto != TouchProto::Single {
        packets.clear();
    }
}

#[inline]
fn retain_multi_b_fingers(
    fingers: &mut FxHashMap<i32, Point>,
    packets: &FxHashMap<i32, TouchState>,
    time: libc::timeval,
    ty: &Sender<DeviceEvent>,
) {
    fingers.retain(|other_id, other_position| {
        packets.contains_key(other_id)
            || ty
                .send(DeviceEvent::Finger {
                    id: *other_id,
                    time: seconds(time),
                    status: FingerStatus::Up,
                    position: *other_position,
                })
                .is_err()
    });
}

fn handle_finger_state_changes(
    fingers: &mut FxHashMap<i32, Point>,
    packets: &FxHashMap<i32, TouchState>,
    time: libc::timeval,
    ty: &Sender<DeviceEvent>,
) {
    for (&id, state) in packets.iter() {
        if let Some(&pos) = fingers.get(&id) {
            if state.pressure > 0 {
                if state.position != pos {
                    send_finger_event(ty, id, time, FingerStatus::Motion, state.position);
                    fingers.insert(id, state.position);
                }
            } else {
                send_finger_event(ty, id, time, FingerStatus::Up, state.position);
                fingers.remove(&id);
            }
        } else if state.pressure > 0 {
            send_finger_event(ty, id, time, FingerStatus::Down, state.position);
            fingers.insert(id, state.position);
        }
    }
}

#[inline]
fn send_finger_event(
    ty: &Sender<DeviceEvent>,
    id: i32,
    time: libc::timeval,
    status: FingerStatus,
    position: Point,
) {
    if ty
        .send(DeviceEvent::Finger {
            id,
            time: seconds(time),
            status,
            position,
        })
        .is_err()
    {
        log_error!("Failed to send finger event");
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_key_event(
    evt: InputEvent,
    button_scheme: &mut ButtonScheme,
    rotation: &mut i8,
    tc: &mut TouchCodes,
    dims: &mut (u32, u32),
    mirror_x: &mut bool,
    mirror_y: &mut bool,
    ty: &Sender<DeviceEvent>,
) {
    if SLEEP_COVER.contains(&evt.code) {
        if evt.value == VAL_PRESS {
            ty.send(DeviceEvent::CoverOn).ok();
        } else if evt.value == VAL_RELEASE {
            ty.send(DeviceEvent::CoverOff).ok();
        } else if evt.value == VAL_REPEAT {
            ty.send(DeviceEvent::CoverOn).ok();
        }
    } else if evt.code == KEY_BUTTON_SCHEME {
        if evt.value == VAL_PRESS {
            *button_scheme = ButtonScheme::Inverted;
        } else {
            *button_scheme = ButtonScheme::Natural;
        }
    } else if evt.code == KEY_ROTATE_DISPLAY {
        let next_rotation = evt.value as i8;
        if next_rotation != *rotation {
            let delta = (*rotation - next_rotation).abs();
            if delta % 2 == 1 {
                mem::swap(&mut tc.x, &mut tc.y);
                mem::swap(&mut dims.0, &mut dims.1);
            }
            *rotation = next_rotation;
            let should_mirror = CURRENT_DEVICE.should_mirror_axes(*rotation);
            *mirror_x = should_mirror.0;
            *mirror_y = should_mirror.1;
        }
    } else if evt.code != BTN_TOUCH {
        if let Some(button_status) = ButtonStatus::try_from_raw(evt.value) {
            if ty
                .send(DeviceEvent::Button {
                    time: seconds(evt.time),
                    code: ButtonCode::from_raw(evt.code, *rotation, *button_scheme),
                    status: button_status,
                })
                .is_err()
            {
                log_error!("Failed to send button event");
            }
        }
    }
}

fn handle_msc_event(evt: InputEvent, ty: &Sender<DeviceEvent>) {
    if evt.value >= MSC_RAW_GSENSOR_PORTRAIT_DOWN && evt.value <= MSC_RAW_GSENSOR_LANDSCAPE_LEFT {
        let next_rotation = GYROSCOPE_ROTATIONS
            .iter()
            .position(|&v| v == evt.value)
            .map(|i| CURRENT_DEVICE.transformed_gyroscope_rotation(i as i8));
        if let Some(next_rotation) = next_rotation {
            ty.send(DeviceEvent::RotateScreen(next_rotation)).ok();
        }
    }
}
