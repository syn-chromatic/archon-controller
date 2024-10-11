#![allow(dead_code)]
#![allow(unused_variables)]

use embsys::crates::defmt;
use embsys::crates::embassy_time;

use embassy_time::Instant;

pub struct FrameTime {
    max: f32,
    min: f32,
    avg: f32,
    count: u32,
    total: f32,
}

impl FrameTime {
    pub fn new() -> Self {
        let min: f32 = f32::MAX;
        let max: f32 = f32::MIN;
        let avg: f32 = 0.0;
        let count: u32 = 0;
        let total: f32 = 0.0;

        Self {
            min,
            max,
            avg,
            count,
            total,
        }
    }

    pub fn update(&mut self, frametime: Instant) {
        let elapsed: f32 = frametime.elapsed().as_millis() as f32;

        if elapsed > self.max {
            self.max = elapsed;
        }

        if elapsed < self.min {
            self.min = elapsed;
        }

        self.total += elapsed;
        self.count += 1;

        self.avg = self.total / self.count as f32;
    }

    pub fn reset(&mut self) {
        self.min = f32::MAX;
        self.max = f32::MIN;
        self.avg = 0.0;
        self.count = 0;
        self.total = 0.0;
    }

    pub fn defmt(&self) {
        defmt::info!(
            "MIN: {:?}ms | MAX: {:?}ms | AVG: {:?}ms",
            self.min,
            self.max,
            self.avg
        );
    }
}
