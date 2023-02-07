use std::fmt::{Display, Formatter};
use std::ops::AddAssign;
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime, Utc};

pub struct StopWatch {
    accumulated: Duration,
    checkpoint: Instant,
    is_stopped: bool,
}

impl StopWatch {
    pub fn new() -> StopWatch {
        StopWatch {
            accumulated: Duration::from_secs(0),
            checkpoint: Instant::now(),
            is_stopped: true,
        }
    }

    pub fn start(&mut self) {
        self.checkpoint = Instant::now();
        self.is_stopped = false;
    }

    pub fn reset(&mut self) {
        self.accumulated = Duration::from_secs(0);
        self.checkpoint = Instant::now();
        self.is_stopped = true
    }

    pub fn restart(&mut self) {
        self.reset();
        self.start();
    }

    pub fn stop(&mut self) {
        self.accumulated.add_assign(Duration::from_nanos(self.checkpoint.elapsed().as_nanos() as u64));
        self.is_stopped = true;
    }
}

impl Display for StopWatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut accumulated = self.accumulated.clone();
        if !self.is_stopped {
            accumulated.add_assign(Duration::from_nanos(self.checkpoint.elapsed().as_nanos() as u64));
        }
        let datetime = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(accumulated.as_secs() as i64, accumulated.subsec_nanos()).unwrap(), Utc);
        let formatted_time = datetime.format("%H:%M:%S.%3f").to_string();
        f.write_fmt(format_args!("{}", formatted_time))
    }
}


#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_stopwatch() {
        // let mut stopwatch = StopWatch::new();
        // assert_eq!("00:00:00.000", stopwatch.to_string());
        // stopwatch.start();
        // thread::sleep(Duration::from_millis(3 ));
        // assert_eq!("00:00:00.003", stopwatch.to_string());
        // thread::sleep(Duration::from_millis(1000 ));
        // assert_eq!("00:00:01.003", stopwatch.to_string());
        // thread::sleep(Duration::from_millis(64038 ));
        // assert_eq!("00:01:05.041", stopwatch.to_string());
    }
}
