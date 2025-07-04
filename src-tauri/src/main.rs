// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod net_monitor;
mod sleep;
use std::sync::{Arc, Mutex};
use tauri::{State, Manager, AppHandle, Emitter};
use net_monitor::NetData;

struct MonitorState {
    running: Arc<Mutex<bool>>,
}

#[tauri::command]
fn start_monitoring(state: State<MonitorState>, app: AppHandle) {
    let running = state.running.clone();
    *running.lock().unwrap() = true;
}

#[tauri::command]
fn stop_monitoring(state: State<MonitorState>) {
    let running = state.running.clone();
    *running.lock().unwrap() = false;
    println!("[DEBUG] stop_monitoring called, running set to false");
}

fn main() {
    tauri::Builder::default()
        .manage(MonitorState {
            running: Arc::new(Mutex::new(true)),
        })
        .invoke_handler(tauri::generate_handler![start_monitoring, stop_monitoring]) 
        .setup(|app| {
            let running = app.state::<MonitorState>().running.clone();
            let window = app.get_webview_window("main").unwrap();
            std::thread::spawn(move || {
                use std::collections::VecDeque;
                let mut is_downloading = "idle".to_string();
                let mut is_downloading_bool = false;
                let mut speeds = VecDeque::with_capacity(5);
                let mut prev_avg = 0.0;
                loop {
                    println!("[DEBUG] Thread loop: running = {}", *running.lock().unwrap());
                    if !*running.lock().unwrap() {
                        println!("[DEBUG] Thread paused.");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    let data = net_monitor::monitor_speed_sample(prev_avg, &mut speeds, &mut is_downloading_bool, &mut is_downloading);
                    println!("[DEBUG] Emitting net-data: avg_kbps = {}, status = {}", data.average_kbps, data.status);
                    window.emit("net-data", data.clone()).unwrap();
                    if data.is_downloading == "false" {
                        println!("[DEBUG] Entering countdown before sleep");
                        for i in (1..7).rev() {
                            if !*running.lock().unwrap() { 
                                println!("[DEBUG] Countdown interrupted by stop");
                                break; 
                            }
                            println!("[DEBUG] Countdown: {}", i);
                            window.emit("net-data", net_monitor::NetData {
                                status: i.to_string(),
                                average_kbps: data.average_kbps,
                                previous_kbps: data.previous_kbps,
                                is_downloading: data.is_downloading.clone(),
                                is_downloading_bool: data.is_downloading_bool,
                            }).unwrap();
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        }
                        if *running.lock().unwrap() {
                            println!("[DEBUG] Calling crate::sleep::sleep()");
                            crate::sleep::sleep();
                        }
                    }
                    prev_avg = data.average_kbps;
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



    

