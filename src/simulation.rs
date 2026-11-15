use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use crate::ant::Ant;
use crate::field::Field;

#[derive(Clone, Default)]
pub struct SimulationSnapshot {
    pub ticks: u64,
    pub actual_ticks_per_second: u32,
    pub season: String,
    pub ants: Vec<Ant>
}

pub struct SimulationWorker {
    is_paused: Arc<AtomicBool>,
    interval_ms: Arc<AtomicU64>,
    wants_snapshot: Arc<AtomicBool>,
    snapshot: Arc<Mutex<SimulationSnapshot>>,
    _handle: thread::JoinHandle<()>
}

impl SimulationWorker {
    pub fn new(width : usize, height: usize) -> Self {

        let is_paused = Arc::new(AtomicBool::new(true));
        let interval_ms = Arc::new(AtomicU64::new(0));
        let wants_snapshot = Arc::new(AtomicBool::new(false));
        let snapshot = Arc::new(Mutex::new(SimulationSnapshot::default()));

        let thread_paused = Arc::clone(&is_paused);
        let thread_interval_ms = Arc::clone(&interval_ms);
        let thread_wants_snapshot = Arc::clone(&wants_snapshot);
        let thread_snapshot = Arc::clone(&snapshot);

        let handle = thread::spawn(move || {
            let mut field: Field = Field::new(width, height);
            let mut steps: u64 = 0;

            let mut last_sec_time = Instant::now();
            let mut ticks_in_current_second = 0;
            let mut actual_tps = 0;

            loop {

                if thread_paused.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                field.tick();
                steps += 1;
                ticks_in_current_second += 1;

                if last_sec_time.elapsed() > Duration::from_secs(1) {
                    actual_tps = ticks_in_current_second;
                    ticks_in_current_second = 0;
                    last_sec_time = Instant::now();
                }

                if thread_wants_snapshot.swap(false, Ordering::Relaxed) {
                    if let Ok(mut state) = thread_snapshot.lock() {
                        state.ticks = steps;
                        state.actual_ticks_per_second = actual_tps;
                        state.season = field.get_season();
                        state.ants = field.get_ants().to_vec();
                        if state.ants.len() == 0 {
                            state.actual_ticks_per_second = 0;
                            break;
                        }
                    }
                }

                let interval = thread_interval_ms.load(Ordering::Relaxed);
                if interval > 0 {
                    thread::sleep(Duration::from_millis(interval));
                }
                else {
                    thread::yield_now();
                }
            }
        });

        Self {
            is_paused,
            interval_ms,
            wants_snapshot,
            snapshot,
            _handle: handle
        }
    }

    pub fn get_snapshot(&self) -> SimulationSnapshot {
        self.wants_snapshot.store(true, Ordering::Relaxed);
        if let Ok(snapshot) = self.snapshot.lock() {
            return snapshot.clone();
        }
        SimulationSnapshot::default()
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused.store(paused, Ordering::Relaxed);
    }

    pub fn set_interval(&mut self, interval: u32) {
        self.interval_ms.store(interval as u64, Ordering::Relaxed);
    }
}