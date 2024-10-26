use embsys::exts::std;

use std::time::Duration;
use std::time::Instant;

pub struct DevicePolling {
    poll_duration: Duration,
    poll_instant: Option<Instant>,
}

impl DevicePolling {
    pub fn new(duration: Duration) -> Self {
        let poll_instant: Option<Instant> = None;
        Self {
            poll_duration: duration,
            poll_instant,
        }
    }

    pub fn poll(&mut self) -> bool {
        if let Some(instant) = self.poll_instant {
            let duration: Duration = instant.elapsed();
            if duration >= self.poll_duration {
                self.poll_instant = Some(Instant::now());
                return true;
            }
        } else {
            self.poll_instant = Some(Instant::now());
            return true;
        }

        false
    }
}
