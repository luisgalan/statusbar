extern crate chrono;
use chrono::Local;

extern crate sys_info;
use sys_info::{cpu_num, loadavg, mem_info};

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Eq, PartialEq)]
enum Task {
    Datetime,
    Battery,
    Memory,
    Load,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Event {
    time: Instant,
    task: Task,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn duration_before(event: &Event) -> Duration {
    event.time.saturating_duration_since(Instant::now())
}

fn get_datetime() -> String {
    let now = Local::now();
    let date = now.format("%b %d %Y (%a)").to_string().to_lowercase();
    let time = now.format("%H:%M:%S").to_string();
    format!("{:17}  {:8}", date, time)
}

fn get_battery() -> Result<String, std::io::Error> {
    let mut file = File::open("/sys/class/power_supply/BAT0/capacity")?;
    let mut battery = String::new();
    file.read_to_string(&mut battery)?;
    Ok(format!("{}%", battery.trim()))
}

fn get_memory() -> Result<String, sys_info::Error> {
    let memory = mem_info()?;
    let percent = 100 * (memory.total - memory.avail) / memory.total;
    Ok(format!("{}%", percent))
}

fn get_load() -> Result<String, sys_info::Error> {
    let load = loadavg()?;
    let num_cores = cpu_num()?;
    let load = (100.0 * load.one / (num_cores as f64)) as u8;
    Ok(format!("{}%", load))
}

fn main() {
    let mut schedule = BinaryHeap::new();
    schedule.push(Event {
        time: Instant::now(),
        task: Task::Memory,
    });
    schedule.push(Event {
        time: Instant::now(),
        task: Task::Load,
    });
    schedule.push(Event {
        time: Instant::now(),
        task: Task::Battery,
    });
    schedule.push(Event {
        time: Instant::now(),
        task: Task::Datetime,
    });

    let mut memory: String = String::new();
    let mut load: String = String::new();
    let mut battery: String = String::new();
    let mut datetime: String = String::new();

    loop {
        // wait for next event and handle it
        sleep(duration_before(schedule.peek().unwrap()));
        while duration_before(schedule.peek().unwrap()) == Duration::from_secs(0) {
            let event = schedule.pop().unwrap();
            match event.task {
                Task::Memory => {
                    if let Ok(res) = get_memory() {
                        memory = res;
                    } else {
                        memory = String::from("???");
                    }
                    schedule.push(Event {
                        time: event.time + Duration::from_secs(1),
                        task: Task::Memory,
                    });
                }
                Task::Load => {
                    if let Ok(res) = get_load() {
                        load = res;
                    } else {
                        memory = String::from("???");
                    }

                    schedule.push(Event {
                        time: event.time + Duration::from_secs(6),
                        task: Task::Load,
                    });
                }
                Task::Battery => {
                    if let Ok(res) = get_battery() {
                        battery = res;
                    } else {
                        memory = String::from("???");
                    }

                    schedule.push(Event {
                        time: event.time + Duration::from_secs(60),
                        task: Task::Battery,
                    });
                }
                Task::Datetime => {
                    datetime = get_datetime();
                    schedule.push(Event {
                        time: event.time + Duration::from_secs(1),
                        task: Task::Datetime,
                    });
                }
            }
        }

        // update status
        let status = format!(
            " ram: {}  cpu: {}  bat: {:3}  {:27} ",
            memory, load, battery, datetime
        );
        let res = Command::new("xsetroot").arg("-name").arg(&status).spawn();
        match res {
            Ok(_) => (),
            Err(e) => eprintln!(
                "error setting root window name. \
                is xsetroot installed? {}",
                e
            ),
        }
    }
}

