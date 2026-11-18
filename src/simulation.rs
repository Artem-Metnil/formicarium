use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use crate::ant::Ant;
use crate::field::Field;

pub struct SimulationConfig {
    pub is_paused: AtomicBool,
    pub interval_ms: AtomicU64,
    pub photosynthesis_multiplier: AtomicU32,
    pub ticks_per_season: AtomicU32,
    pub corpse_energy_ratio: AtomicU32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            is_paused: AtomicBool::new(true),
            interval_ms: AtomicU64::new(10),
            photosynthesis_multiplier: AtomicU32::new(1.0f32.to_bits()),
            ticks_per_season: AtomicU32::new(0),
            corpse_energy_ratio: AtomicU32::new(0.5f32.to_bits()),
        }
    }
}

#[derive(Clone, Default)]
pub struct SimulationSnapshot {
    pub ticks: u64,
    pub actual_ticks_per_second: u32,
    pub season: String,
    pub ants: Vec<Ant>
}

impl SimulationSnapshot {
    pub fn object_count(&self) -> usize {
        self.ants.len()
    }
}

pub struct SimulationWorker {
    config: Arc<SimulationConfig>,
    wants_snapshot: Arc<AtomicBool>,
    snapshot: Arc<Mutex<SimulationSnapshot>>,
    _handle: thread::JoinHandle<()>
}

impl SimulationWorker {
    pub fn new(width : usize, height: usize) -> Self {

        let config = Arc::new(SimulationConfig::default());
        let wants_snapshot = Arc::new(AtomicBool::new(false));
        let snapshot = Arc::new(Mutex::new(SimulationSnapshot::default()));

        let thread_config = Arc::clone(&config);
        let thread_wants_snapshot = Arc::clone(&wants_snapshot);
        let thread_snapshot = Arc::clone(&snapshot);

        let handle = thread::spawn(move || {
            let mut field: Field = Field::new(width, height);
            let mut steps: u64 = 0;

            let mut last_sec_time = Instant::now();
            let mut ticks_in_current_second = 0;
            let mut actual_tps = 0;

            loop {

                if thread_config.is_paused.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                field.tick(&thread_config);
                steps += 1;
                ticks_in_current_second += 1;

                if field.finished() {
                    if let Ok(mut state) = thread_snapshot.lock() {
                        state.ticks = steps;
                        state.actual_ticks_per_second = 0;
                        state.season = field.get_season();
                        state.ants = field.get_ants().to_vec();
                    }
                    break;
                }

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

                let interval = thread_config.interval_ms.load(Ordering::Relaxed);
                if interval > 0 {
                    thread::sleep(Duration::from_millis(interval));
                }
                else {
                    thread::yield_now();
                }
            }
        });

        Self {
            config,
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
        self.config.is_paused.store(paused, Ordering::Relaxed);
    }

    pub fn set_interval(&mut self, interval: u32) {
        self.config.interval_ms.store(interval as u64, Ordering::Relaxed);
    }

    pub fn set_photosynthesis_multiplier(&self, val: f32) {
        self.config.photosynthesis_multiplier.store(val.to_bits(), Ordering::Relaxed);
    }
    pub fn set_ticks_per_season(&self, val: u32) {
        self.config.ticks_per_season.store(val, Ordering::Relaxed);
    }
    pub fn set_corpse_energy_ratio(&self, val: f32) {
        self.config.corpse_energy_ratio.store(val.to_bits(), Ordering::Relaxed);
    }
}