//! System Information HTML Generator
//!
//! Generates HTML-formatted system information for display in the reader.
//! Collects device details, memory stats, storage info, and hardware configuration.

use crate::device::CURRENT_DEVICE;
use crate::document::HumanSize;
use crate::log_error;
#[cfg(target_os = "linux")]
use nix::sys::statvfs;
#[cfg(target_os = "linux")]
use nix::sys::sysinfo;
use regex::Regex;
use rustc_hash::FxHashMap;
use std::env;
use std::fs;
use std::process::Command;

const INTERNAL_CARD_ROOT: &str = "/mnt/onboard";

const CPUINFO_KEYS: [&str; 3] = ["Processor", "Features", "Hardware"];

const HWINFO_KEYS: [&str; 19] = [
    "CPU",
    "PCB",
    "DisplayPanel",
    "DisplayCtrl",
    "DisplayBusWidth",
    "DisplayResolution",
    "FrontLight",
    "FrontLight_LEDrv",
    "FL_PWM",
    "TouchCtrl",
    "TouchType",
    "Battery",
    "IFlash",
    "RamSize",
    "RamType",
    "LightSensor",
    "HallSensor",
    "RSensor",
    "Wifi",
];

/// Generate system information as HTML
///
/// Collects and formats device model, hardware info, memory status,
/// storage capacity, CPU details, and hardware configuration.
pub fn sys_info_as_html() -> String {
    let mut buf = "<html>\n\t<head>\n\t\t<title>System Info</title>\n\t\t\
                   <link rel=\"stylesheet\" type=\"text/css\" \
                   href=\"css/sysinfo.css\"/>\n\t</head>\n\t<body>\n"
        .to_string();

    buf.push_str("\t\t<table>\n");

    // Device model info
    buf.push_str("\t\t\t<tr>\n");
    buf.push_str("\t\t\t\t<td class=\"key\">Model name</td>\n");
    buf.push_str(&format!(
        "\t\t\t\t<td class=\"value\">{}</td>\n",
        CURRENT_DEVICE.model
    ));
    buf.push_str("\t\t\t</tr>\n");

    buf.push_str("\t\t\t<tr>\n");
    buf.push_str("\t\t\t\t<td class=\"key\">Hardware</td>\n");
    buf.push_str(&format!(
        "\t\t\t\t<td class=\"value\">Mark {}</td>\n",
        CURRENT_DEVICE.mark()
    ));
    buf.push_str("\t\t\t</tr>\n");
    buf.push_str("\t\t\t<tr class=\"sep\"></tr>\n");

    // Environment variables
    for (name, var) in [
        ("Code name", "PRODUCT"),
        ("Model number", "MODEL_NUMBER"),
        ("Firmware version", "FIRMWARE_VERSION"),
    ]
    .iter()
    {
        if let Ok(value) = env::var(var) {
            buf.push_str("\t\t\t<tr>\n");
            buf.push_str(&format!("\t\t\t\t<td class=\"key\">{}</td>\n", name));
            buf.push_str(&format!("\t\t\t\t<td class=\"value\">{}</td>\n", value));
            buf.push_str("\t\t\t</tr>\n");
        }
    }

    buf.push_str("\t\t\t<tr class=\"sep\"></tr>\n");

    // IP Address
    append_ip_address(&mut buf);

    // Storage info
    append_storage_info(&mut buf);

    // Memory info (Linux only)
    #[cfg(target_os = "linux")]
    append_memory_info(&mut buf);
    #[cfg(target_os = "ios")]
    {
        // iOS memory info not yet implemented
    }

    // CPU info
    append_cpu_info(&mut buf);

    // Hardware config
    append_hwconfig_info(&mut buf);

    buf.push_str("\t\t</table>\n\t</body>\n</html>");
    buf
}

fn append_ip_address(buf: &mut String) {
    let output = Command::new("scripts/ip.sh")
        .output()
        .map_err(|e| log_error!("Can't execute command: {:#}.", e))
        .ok();

    if let Some(stdout) = output
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .filter(|stdout| !stdout.is_empty())
    {
        buf.push_str("\t\t\t<tr>\n");
        buf.push_str("\t\t\t\t<td>IP Address</td>\n");
        buf.push_str(&format!("\t\t\t\t<td>{}</td>\n", stdout));
        buf.push_str("\t\t\t</tr>\n");
    }
}

fn append_storage_info(buf: &mut String) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(info) = statvfs::statvfs(INTERNAL_CARD_ROOT) {
            let fbs = info.fragment_size();
            let free = info.blocks_free() * fbs;
            let total = info.blocks() * fbs;
            buf.push_str("\t\t\t<tr>\n");
            buf.push_str("\t\t\t\t<td>Storage (Free / Total)</td>\n");
            buf.push_str(&format!(
                "\t\t\t\t<td>{} / {}</td>\n",
                free.human_size(),
                total.human_size()
            ));
            buf.push_str("\t\t\t</tr>\n");
        }
    }
    #[cfg(target_os = "ios")]
    {
        // iOS storage info - use Foundation framework
        buf.push_str("\t\t\t<tr>\n");
        buf.push_str("\t\t\t\t<td>Storage</td>\n");
        buf.push_str("\t\t\t\t<td>Available on iOS</td>\n");
        buf.push_str("\t\t\t</tr>\n");
    }
}

#[cfg(target_os = "linux")]
fn append_memory_info(buf: &mut String) {
    if let Ok(info) = sysinfo::sysinfo() {
        buf.push_str("\t\t\t<tr>\n");
        buf.push_str("\t\t\t\t<td>Memory (Free / Total)</td>\n");
        buf.push_str(&format!(
            "\t\t\t\t<td>{} / {}</td>\n",
            info.ram_unused().human_size(),
            info.ram_total().human_size()
        ));
        buf.push_str("\t\t\t</tr>\n");

        let load = info.load_average();
        buf.push_str("\t\t\t<tr>\n");
        buf.push_str("\t\t\t\t<td>Load Average</td>\n");
        buf.push_str(&format!(
            "\t\t\t\t<td>{:.1}% {:.1}% {:.1}%</td>\n",
            load.0 * 100.0,
            load.1 * 100.0,
            load.2 * 100.0
        ));
        buf.push_str("\t\t\t</tr>\n");
    }
}

fn append_cpu_info(buf: &mut String) {
    buf.push_str("\t\t\t<tr class=\"sep\"></tr>\n");

    if let Ok(info) = fs::read_to_string("/proc/cpuinfo") {
        for line in info.lines() {
            if let Some(index) = line.find(':') {
                let key = line[0..index].trim();
                let value = line[index + 1..].trim();
                if CPUINFO_KEYS.contains(&key) {
                    buf.push_str("\t\t\t<tr>\n");
                    buf.push_str(&format!("\t\t\t\t<td class=\"key\">{}</td>\n", key));
                    buf.push_str(&format!("\t\t\t\t<td class=\"value\">{}</td>\n", value));
                    buf.push_str("\t\t\t</tr>\n");
                }
            }
        }
    }
}

fn append_hwconfig_info(buf: &mut String) {
    buf.push_str("\t\t\t<tr class=\"sep\"></tr>\n");

    let output = Command::new("/bin/ntx_hwconfig")
        .args(["-s", "/dev/mmcblk0"])
        .output()
        .map_err(|e| log_error!("Can't execute command: {:#}.", e))
        .ok();

    let mut map = FxHashMap::default();

    if let Some(stdout) = output.and_then(|output| String::from_utf8(output.stdout).ok()) {
        if let Ok(re) = Regex::new(r"\[\d+\]\s+(?P<key>[^=]+)='(?P<value>[^']+)'") {
            for caps in re.captures_iter(&stdout) {
                map.insert(caps["key"].to_string(), caps["value"].to_string());
            }
        }
    }

    if !map.is_empty() {
        let mut row_buf = String::with_capacity(128);
        for key in HWINFO_KEYS.iter() {
            if let Some(value) = map.get(*key) {
                row_buf.clear();
                row_buf.push_str("\t\t\t<tr>\n");
                row_buf.push_str("\t\t\t\t<td>");
                row_buf.push_str(key);
                row_buf.push_str("</td>\n");
                row_buf.push_str("\t\t\t\t<td>");
                row_buf.push_str(value);
                row_buf.push_str("</td>\n");
                row_buf.push_str("\t\t\t</tr>\n");
                buf.push_str(&row_buf);
            }
        }
    }
}
