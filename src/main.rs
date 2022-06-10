mod config;

#[macro_export]
macro_rules! make_statusbar {
    ($(task $task_idents:ident: $task_types:ty $task_blocks:block),*, status $status:block) => {
        #[derive(Copy, Clone, Eq, PartialEq)]
        #[allow(non_camel_case_types)]
        enum Task {
            $($task_idents),*
        }

        #[derive(Copy, Clone, Eq, PartialEq)]
        struct Event {
            time: std::time::Instant,
            task: Task,
        }

        impl Ord for Event {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other.time.cmp(&self.time)
            }
        }

        impl PartialOrd for Event {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        pub fn main_loop() {
            use std::collections::BinaryHeap;
            use std::thread::sleep;
            use x11rb::connection::Connection;
            use x11rb::protocol::xproto::*;
            use x11rb::wrapper::ConnectionExt as _;

            fn duration_before(event: &Event) -> Duration {
                event.time.saturating_duration_since(Instant::now())
            }

            let (conn, screen_num) = x11rb::connect(None).unwrap();
            let root_window = &conn.setup().roots[screen_num].root;

            let mut schedule = BinaryHeap::new();

            $(
                let mut $task_idents: $task_types;
            )*

            $(
                schedule.push(Event {
                    time: $task_blocks,
                    task: Task::$task_idents,
                });
            )*

            loop {
                // update status
                let status = $status;
                conn.change_property8(
                    PropMode::REPLACE,
                    *root_window,
                    AtomEnum::WM_NAME,
                    AtomEnum::STRING,
                    status.as_bytes(),
                )
                .unwrap();
                conn.flush().unwrap();

                // wait for next event
                sleep(duration_before(&schedule.peek().unwrap()));

                // handle due events
                while duration_before(schedule.peek().unwrap()) == Duration::from_secs(0) {
                    let event = schedule.pop().unwrap();
                    match event.task {
                        $(
                            Task::$task_idents => {
                                schedule.push(Event {
                                    time: $task_blocks,
                                    task: Task::$task_idents
                                });
                            }
                        ),*
                    };
                }
            }
        }
    };
}

fn main() {
    config::main_loop();
}
