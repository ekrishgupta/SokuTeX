use std::time::Instant;

pub struct PerfTimer {
    start: Instant,
    label: String,
}

impl PerfTimer {
    pub fn start(label: &str) -> Self {
        Self {
            start: Instant::now(),
            label: label.to_string(),
        }
    }

    pub fn stop(self) {
        let duration = self.start.elapsed();
        println!("[PERF] {}: {:?}", self.label, duration);
    }
}
