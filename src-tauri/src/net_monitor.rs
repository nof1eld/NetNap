use std::{collections::VecDeque, thread, time::Duration};

#[cfg(target_os = "windows")]
use windows::Win32::NetworkManagement::IpHelper::{GetIfTable2, FreeMibTable, MIB_IF_TABLE2, MIB_IF_ROW2};
#[cfg(target_os = "windows")]
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::NO_ERROR;
#[cfg(target_os = "windows")]
use std::ptr;

#[cfg(target_os = "linux")]
use std::{fs::File, io::{BufRead, BufReader}};

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct NetData {
    pub average_kbps: f64,
    pub previous_kbps: f64,
    pub is_downloading: String,
    pub is_downloading_bool: bool,
    pub status: String,
}

#[cfg(target_os = "windows")]
fn get_total_rx_bytes() -> u64 {
    unsafe {
        let mut table_ptr: *mut MIB_IF_TABLE2 = ptr::null_mut();
        let mut total_rx: u64 = 0;
        if GetIfTable2(&mut table_ptr) == NO_ERROR && !table_ptr.is_null() {
            let table = &*table_ptr;
            for i in 0..table.NumEntries as usize {
                let row: &MIB_IF_ROW2 = &*table.Table.as_ptr().add(i);
                if row.PhysicalAddressLength == 0 || row.OperStatus != IfOperStatusUp {
                    continue;
                }
                total_rx += row.InOctets;
            }
            FreeMibTable(table_ptr as _);
        }
        total_rx
    }
}

#[cfg(target_os = "linux")]
fn get_total_rx_bytes() -> u64 {
    let file = File::open("/proc/net/dev").expect("Failed to open /proc/net/dev");
    let reader = BufReader::new(file);
    let mut total_rx: u64 = 0;
    for line in reader.lines().skip(2) {
        if let Ok(line) = line {
            if let Some(colon) = line.find(':') {
                let stats = line[(colon + 1)..].split_whitespace().collect::<Vec<_>>();
                if let Some(rx_bytes_str) = stats.get(0) {
                    if let Ok(rx_bytes) = rx_bytes_str.parse::<u64>() {
                        total_rx += rx_bytes;
                    }
                }
            }
        }
    }
    total_rx
}

fn get_kbps() -> f64 {
    let prev_total_rx = get_total_rx_bytes();
    thread::sleep(Duration::from_secs(1));
    let current_total_rx = get_total_rx_bytes();
    let delta = current_total_rx.saturating_sub(prev_total_rx);
    delta as f64 / 1024.0
}

fn get_average_speed(speeds: &mut VecDeque<f64>) -> f64 {
    if speeds.len() == speeds.capacity() {
        speeds.pop_front();
    }
    speeds.push_back(get_kbps());
    speeds.iter().copied().sum::<f64>() / speeds.len() as f64
}

pub fn monitor_speed_sample(prev_avg: f64, speeds: &mut VecDeque<f64>, is_downloading_bool: &mut bool, is_downloading: &mut String) -> NetData {
    let avg = get_average_speed(speeds);
    let mut status = String::from("Stable");
    // Determine state transitions
    if !*is_downloading_bool {
        if avg > 3.0 * prev_avg && avg > 100.0 {
            *is_downloading = "true".to_string();
            *is_downloading_bool = true;
            status = String::from("Download started");
        } else {
            *is_downloading = "idle".to_string();
            status = String::from("Stable");
        }
    } else {
        if avg < prev_avg / 3.0 {
            *is_downloading = "false".to_string();
            *is_downloading_bool = false;
            status = String::from("Download stopped. Sleeping soon...");
        } else {
            *is_downloading = "true".to_string();
            status = String::from("Downloading...");
        }
    }
    NetData {
        average_kbps: avg,
        previous_kbps: prev_avg,
        is_downloading: is_downloading.clone(),
        is_downloading_bool: *is_downloading_bool,
        status: status.clone(),
    }
}
