use crate::make_statusbar;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::prelude::*;

extern crate chrono;
use chrono::Local;

extern crate sys_info;
use sys_info::{cpu_num, loadavg, mem_info};

fn get_datetime() -> String {
    let now = Local::now();
    let date = now.format("%b %d %Y (%a)").to_string().to_lowercase();
    let time = now.format("%H:%M:%S").to_string();
    format!("{:17}  {:8}", date, time)
}

fn get_battery() -> Result<String, std::io::Error> {
    let mut file = File::open("sys/class/power_supply/BAT0/capacity")?;
    let mut battery = String::new();
    file.read_to_string(&mut battery)?;
    Ok(format!("{}%", battery.trim()))
}

fn get_memory() -> Result<String, sys_info::Error> {
    let memory = mem_info()?;
    let percent = 100 * (memory.total - memory.avail) / memory.total;
    Ok(format!("{}%", percent))
}

fn get_cpu_load() -> Result<String, sys_info::Error> {
    let load = loadavg()?;
    let num_cores = cpu_num()?;
    let load = (100.0 * load.one / (num_cores as f64)) as u8;
    Ok(format!("{}%", load))
}

make_statusbar! {
    task time: String {
        time = get_datetime();
        Instant::now() + Duration::from_secs(1)
    },
    task battery_info: String {
        battery_info = get_battery().unwrap_or_else(|_| String::from("???"));
        Instant::now() + Duration::from_secs(60)
    },
    task memory_info: String {
        memory_info = get_memory().unwrap_or_else(|_| String::from("???"));
        Instant::now() + Duration::from_secs(1)
    },
    task cpu_load_info: String {
        cpu_load_info = get_cpu_load().unwrap_or_else(|_| String::from("???"));
        Instant::now() + Duration::from_secs(6)
    },
    status {
        format!(" ram: {}  cpu: {}  bat: {}  {:27} ", memory_info, cpu_load_info, battery_info, time)
    }
}

